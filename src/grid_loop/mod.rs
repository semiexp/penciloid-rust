use super::{FiniteSearchQueue, Grid, D, LP, P};
use std::iter::IntoIterator;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use std::mem;
use FOUR_NEIGHBOURS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Undecided,
    Line,
    Blank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EdgeId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VtxId(usize);

#[derive(Clone, Copy)]
struct GridLoopItem {
    edge_status: Edge,
    chain_end_points: (VtxId, VtxId),
    chain_next: EdgeId,
    chain_another_end_edge: EdgeId,
    chain_size: i32,
}

#[derive(Clone)]
pub struct GridLoop {
    grid: Grid<GridLoopItem>,
    inconsistent: bool,
    fully_solved: bool,
    decided_line: i32,
    decided_edge: i32,
    queue: FiniteSearchQueue,
}
impl Index<EdgeId> for GridLoop {
    type Output = GridLoopItem;
    fn index(&self, index: EdgeId) -> &GridLoopItem {
        &self.grid[index.0]
    }
}
impl IndexMut<EdgeId> for GridLoop {
    fn index_mut(&mut self, index: EdgeId) -> &mut GridLoopItem {
        &mut self.grid[index.0]
    }
}
impl GridLoop {
    pub fn new(height: i32, width: i32) -> GridLoop {
        let mut grid = Grid::new(
            height * 2 + 1,
            width * 2 + 1,
            GridLoopItem {
                edge_status: Edge::Undecided,
                chain_end_points: (VtxId(0), VtxId(0)),
                chain_next: EdgeId(0),
                chain_another_end_edge: EdgeId(0),
                chain_size: 0,
            },
        );

        for y in 0..(height * 2 + 1) {
            for x in 0..(width * 2 + 1) {
                if y % 2 == x % 2 {
                    continue;
                }
                let pos = LP(y, x);
                let id = grid.index_lp(pos);
                grid[pos] = GridLoopItem {
                    edge_status: Edge::Undecided,
                    chain_end_points: if y % 2 == 0 {
                        (VtxId(id - 1), VtxId(id + 1))
                    } else {
                        (
                            VtxId(id - (width * 2 + 1) as usize),
                            VtxId(id + (width * 2 + 1) as usize),
                        )
                    },
                    chain_next: EdgeId(id),
                    chain_another_end_edge: EdgeId(id),
                    chain_size: 1,
                };
            }
        }

        let mut ret = GridLoop {
            grid: grid,
            inconsistent: false,
            fully_solved: false,
            decided_line: 0,
            decided_edge: 0,
            queue: FiniteSearchQueue::new(((height * 2 + 1) * (width * 2 + 1)) as usize),
        };

        ret.queue.start();
        {
            let edge1 = ret.grid.index_lp(LP(0, 1));
            let edge2 = ret.grid.index_lp(LP(1, 0));
            GridLoop::join(&mut ret, EdgeId(edge1), EdgeId(edge2));
        }
        {
            let edge1 = ret.grid.index_lp(LP(0, width * 2 - 1));
            let edge2 = ret.grid.index_lp(LP(1, width * 2));
            GridLoop::join(&mut ret, EdgeId(edge1), EdgeId(edge2));
        }
        {
            let edge1 = ret.grid.index_lp(LP(height * 2 - 1, 0));
            let edge2 = ret.grid.index_lp(LP(height * 2, 1));
            GridLoop::join(&mut ret, EdgeId(edge1), EdgeId(edge2));
        }
        {
            let edge1 = ret.grid.index_lp(LP(height * 2 - 1, width * 2));
            let edge2 = ret.grid.index_lp(LP(height * 2, width * 2 - 1));
            GridLoop::join(&mut ret, EdgeId(edge1), EdgeId(edge2));
        }
        GridLoop::queue_pop_all(&mut ret);
        ret.queue.finish();

        ret
    }

    // public accessor
    pub fn height(&self) -> i32 {
        self.grid.height() / 2
    }
    pub fn width(&self) -> i32 {
        self.grid.width() / 2
    }
    pub fn inconsistent(&self) -> bool {
        self.inconsistent
    }
    pub fn fully_solved(&self) -> bool {
        self.fully_solved
    }
    pub fn get_edge(&self, pos: LP) -> Edge {
        self.grid[pos].edge_status
    }
    pub fn get_edge_safe(&self, pos: LP) -> Edge {
        if self.is_valid_lp(pos) {
            self.get_edge(pos)
        } else {
            Edge::Blank
        }
    }
    pub fn is_valid_lp(&self, pos: LP) -> bool {
        0 <= pos.0 && pos.0 < self.grid.height() && 0 <= pos.1 && pos.1 < self.grid.width()
    }
    pub fn is_vertex(&self, pos: LP) -> bool {
        pos.is_vertex()
    }
    pub fn is_edge(&self, pos: LP) -> bool {
        pos.is_edge()
    }
    pub fn num_decided_edges(&self) -> i32 {
        self.decided_edge
    }
    pub fn num_decided_lines(&self) -> i32 {
        self.decided_line
    }
    pub fn neighbor_summary(&self, pos: LP) -> (i32, i32) {
        let mut n_line = 0;
        let mut n_undecided = 0;
        for &d in &FOUR_NEIGHBOURS {
            let e = self.get_edge_safe(pos + d);
            if e == Edge::Line {
                n_line += 1;
            } else if e == Edge::Undecided {
                n_undecided += 1;
            }
        }
        (n_line, n_undecided)
    }

    // public modifier
    pub fn set_inconsistent(&mut self) {
        self.inconsistent = true;
    }
    pub fn decide_edge<T: GridLoopField>(field: &mut T, pos: LP, status: Edge) {
        if !field.grid_loop().is_valid_lp(pos) {
            if status != Edge::Blank {
                field.grid_loop().inconsistent = true;
            }
            return;
        }

        let id = field.grid_loop().grid.index_lp(pos);
        let current_status = field.grid_loop().grid[id].edge_status;

        if current_status == status {
            return;
        }
        if current_status != Edge::Undecided {
            field.grid_loop().inconsistent = true;
            return;
        }

        let mut handle = GridLoop::get_handle(field);
        GridLoop::decide_edge_internal(&mut *handle, EdgeId(id), status);
    }
    pub fn check<T: GridLoopField>(field: &mut T, pos: LP) {
        if !field.grid_loop().is_valid_lp(pos) {
            return;
        }

        let id = field.grid_loop().grid.index_lp(pos);
        let mut handle = GridLoop::get_handle(field);
        handle.grid_loop().queue.push(id);
    }
    pub fn get_handle<'a, T: GridLoopField>(field: &'a mut T) -> QueueActiveGridLoopField<'a, T> {
        QueueActiveGridLoopField::new(field)
    }
    pub fn apply_inout_rule<T: GridLoopField>(field: &mut T) {
        let height = field.grid_loop().height();
        let width = field.grid_loop().width();
        let mut side = Grid::new(height, width, -1);
        let mut handle = GridLoop::get_handle(field);

        // outside the field
        for x in 0..width {
            let edge = handle.grid_loop().get_edge(LP(0, 2 * x + 1));
            if edge == Edge::Blank {
                GridLoop::apply_inout_rule_dfs(P(0, x), 0, &mut *handle, &mut side);
            } else if edge == Edge::Line {
                GridLoop::apply_inout_rule_dfs(P(0, x), 1, &mut *handle, &mut side);
            }

            let edge = handle.grid_loop().get_edge(LP(2 * height, 2 * x + 1));
            if edge == Edge::Blank {
                GridLoop::apply_inout_rule_dfs(P(height - 1, x), 0, &mut *handle, &mut side);
            } else if edge == Edge::Line {
                GridLoop::apply_inout_rule_dfs(P(height - 1, x), 1, &mut *handle, &mut side);
            }
        }
        for y in 0..height {
            let edge = handle.grid_loop().get_edge(LP(2 * y + 1, 0));
            if edge == Edge::Blank {
                GridLoop::apply_inout_rule_dfs(P(y, 0), 0, &mut *handle, &mut side);
            } else if edge == Edge::Line {
                GridLoop::apply_inout_rule_dfs(P(y, 0), 1, &mut *handle, &mut side);
            }

            let edge = handle.grid_loop().get_edge(LP(2 * y + 1, 2 * width));
            if edge == Edge::Blank {
                GridLoop::apply_inout_rule_dfs(P(y, width - 1), 0, &mut *handle, &mut side);
            } else if edge == Edge::Line {
                GridLoop::apply_inout_rule_dfs(P(y, width - 1), 1, &mut *handle, &mut side);
            }
        }
        for x in 0..width {
            if side[P(0, x)] == 0 {
                GridLoop::decide_edge(&mut *handle, LP(0, 2 * x + 1), Edge::Blank);
            } else if side[P(0, x)] == 1 {
                GridLoop::decide_edge(&mut *handle, LP(0, 2 * x + 1), Edge::Line);
            }

            if side[P(height - 1, x)] == 0 {
                GridLoop::decide_edge(&mut *handle, LP(2 * height, 2 * x + 1), Edge::Blank);
            } else if side[P(height - 1, x)] == 1 {
                GridLoop::decide_edge(&mut *handle, LP(2 * height, 2 * x + 1), Edge::Line);
            }
        }
        for y in 0..height {
            if side[P(y, 0)] == 0 {
                GridLoop::decide_edge(&mut *handle, LP(2 * y + 1, 0), Edge::Blank);
            } else if side[P(y, 0)] == 1 {
                GridLoop::decide_edge(&mut *handle, LP(2 * y + 1, 0), Edge::Line);
            }

            if side[P(y, width - 1)] == 0 {
                GridLoop::decide_edge(&mut *handle, LP(2 * y + 1, 2 * width), Edge::Blank);
            } else if side[P(y, width - 1)] == 1 {
                GridLoop::decide_edge(&mut *handle, LP(2 * y + 1, 2 * width), Edge::Line);
            }
        }

        let mut id = 2;
        for y in 0..height {
            for x in 0..width {
                let pos = P(y, x);
                if side[pos] == -1 {
                    GridLoop::apply_inout_rule_dfs(pos, id, &mut *handle, &mut side);
                    id += 2;
                }
            }
        }
    }
    pub fn check_connectability<T: GridLoopField>(field: &mut T) {
        let grid = field.grid_loop();
        let height = grid.height();
        let width = grid.width();
        let mut visited = Grid::new(height + 1, width + 1, false);

        for y in 0..(2 * height + 1) {
            for x in 0..(2 * width + 1) {
                if y % 2 != x % 2 && grid.get_edge(LP(y, x)) == Edge::Line {
                    let mut n_lines_dbl = 0;
                    grid.check_connectability_dfs(P(y / 2, x / 2), &mut visited, &mut n_lines_dbl);

                    if n_lines_dbl > 0 {
                        if n_lines_dbl != grid.num_decided_lines() * 2 {
                            grid.set_inconsistent();
                        }
                        return;
                    }
                }
            }
        }
    }
    fn apply_inout_rule_dfs<T: GridLoopField>(pos: P, v: i32, field: &mut T, side: &mut Grid<i32>) {
        if side[pos] != -1 {
            return;
        }
        side[pos] = v;

        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            if side.is_valid_p(pos2) {
                let v2 = side[pos2];
                if v2 == -1 {
                    let edge = field.grid_loop().get_edge(LP::of_cell(pos) + d);
                    if edge != Edge::Undecided {
                        GridLoop::apply_inout_rule_dfs(
                            pos2,
                            if edge == Edge::Line { v ^ 1 } else { v },
                            field,
                            side,
                        );
                    }
                } else if (v & !1) == (v2 & !1) {
                    GridLoop::decide_edge(
                        field,
                        LP::of_cell(pos) + d,
                        if v2 == v { Edge::Blank } else { Edge::Line },
                    );
                }
            }
        }
    }
    fn check_connectability_dfs(&self, pos: P, visited: &mut Grid<bool>, n_lines_dbl: &mut i32) {
        if visited[pos] {
            return;
        }
        visited[pos] = true;

        let dy = [1, 0, -1, 0];
        let dx = [0, 1, 0, -1];
        for &d in &FOUR_NEIGHBOURS {
            let pos2 = pos + d;
            if visited.is_valid_p(pos2) {
                let edge = self.get_edge(LP::of_vertex(pos) + d);
                if edge == Edge::Blank {
                    continue;
                }
                if edge == Edge::Line {
                    *n_lines_dbl += 1;
                }
                self.check_connectability_dfs(pos2, visited, n_lines_dbl);
            }
        }
    }

    pub fn is_root(&self, edge: LP) -> bool {
        let id = EdgeId(self.grid.index_lp(edge));
        let id2 = self[id].chain_another_end_edge;
        self[id2].chain_another_end_edge == id && id.0 <= id2.0
    }

    // private accessor
    fn another_end_id(&self, origin: VtxId, edge: EdgeId) -> VtxId {
        let edge_data = self[edge];
        VtxId((edge_data.chain_end_points.0).0 + (edge_data.chain_end_points.1).0 - origin.0)
    }
    fn is_end_of_chain(&self, id: EdgeId) -> bool {
        let id2 = self[id].chain_another_end_edge;
        self[id2].chain_another_end_edge == id
    }
    fn is_end_of_chain_vertex(&self, edge: EdgeId, vtx: VtxId) -> bool {
        let ends = self[edge].chain_end_points;
        ends.0 == vtx || ends.1 == vtx
    }

    // private modifier
    fn queue_pop_all<T: GridLoopField>(field: &mut T) {
        while !field.grid_loop().queue.empty() {
            let id = field.grid_loop().queue.pop();
            if field.grid_loop().inconsistent() {
                continue;
            }
            let pos = field.grid_loop().grid.lp(id);
            field.inspect(pos);
            if field.grid_loop().is_vertex(pos) {
                GridLoop::inspect_vertex(field, pos);
            }
        }
    }
    fn decide_edge_internal<T: GridLoopField>(field: &mut T, id: EdgeId, status: Edge) {
        let current_status = field.grid_loop()[id].edge_status;

        if current_status == status {
            return;
        }
        if current_status != Edge::Undecided {
            field.grid_loop().inconsistent = true;
            return;
        }

        GridLoop::decide_chain(field, id, status);
        GridLoop::check_chain_neighborhood(field, id);
    }
    fn decide_chain<T: GridLoopField>(field: &mut T, edge: EdgeId, status: Edge) {
        let gl = field.grid_loop();
        let mut pt = edge;
        let mut sz = 0;
        loop {
            gl[pt].edge_status = status;
            pt = gl[pt].chain_next;
            sz += 1;
            if pt == edge {
                break;
            }
        }
        gl.decided_edge += sz;
        if status == Edge::Line {
            gl.decided_line += sz;
        }
    }
    fn check_chain_neighborhood<T: GridLoopField>(field: &mut T, edge: EdgeId) {
        let mut pt = edge;
        loop {
            let pos = field.grid_loop().grid.lp(pt.0);
            field.check_neighborhood(pos);
            pt = field.grid_loop()[pt].chain_next;
            if pt == edge {
                break;
            }
        }
    }
    fn has_fully_solved<T: GridLoopField>(field: &mut T) {
        let height = field.grid_loop().height();
        let width = field.grid_loop().width();
        for y in 0..(2 * height + 1) {
            for x in 0..(2 * width + 1) {
                if y % 2 != x % 2 && field.grid_loop().get_edge(LP(y, x)) == Edge::Undecided {
                    GridLoop::decide_edge(field, LP(y, x), Edge::Blank);
                }
            }
        }
    }
    fn join<T: GridLoopField>(field: &mut T, edge1: EdgeId, edge2: EdgeId) {
        let mut item1 = field.grid_loop()[edge1];
        let mut item2 = field.grid_loop()[edge2];

        if !field.grid_loop().is_end_of_chain(edge1) || !field.grid_loop().is_end_of_chain(edge2) {
            return;
        }
        if item1.chain_another_end_edge == edge2 {
            return;
        }

        // ensure item1.0 == item2.0
        match (item1.chain_end_points, item2.chain_end_points) {
            ((ex, _), (ey, _)) if ex == ey => (),
            ((ex, _), (_, ey)) if ex == ey => {
                mem::swap(&mut item2.chain_end_points.0, &mut item2.chain_end_points.1)
            }
            ((_, ex), (ey, _)) if ex == ey => {
                mem::swap(&mut item1.chain_end_points.0, &mut item1.chain_end_points.1)
            }
            ((_, ex), (_, ey)) if ex == ey => {
                mem::swap(&mut item1.chain_end_points.0, &mut item1.chain_end_points.1);
                mem::swap(&mut item2.chain_end_points.0, &mut item2.chain_end_points.1);
            }
            _ => return,
        }

        let origin = item1.chain_end_points.0;
        let end1_vertex = field.grid_loop().another_end_id(origin, edge1);
        let end2_vertex = field.grid_loop().another_end_id(origin, edge2);
        let end1_edge = field.grid_loop()[edge1].chain_another_end_edge;
        let end2_edge = field.grid_loop()[edge2].chain_another_end_edge;
        let status;

        match (
            field.grid_loop()[edge1].edge_status,
            field.grid_loop()[edge2].edge_status,
        ) {
            (status1, status2) if status1 == status2 => status = status1,
            (Edge::Undecided, status2) => {
                GridLoop::decide_chain(field, edge1, status2);
                GridLoop::check_chain_neighborhood(field, edge1);
                GridLoop::join(field, edge1, edge2);
                return;
            }
            (status1, Edge::Undecided) => {
                GridLoop::decide_chain(field, edge2, status1);
                GridLoop::check_chain_neighborhood(field, edge2);
                GridLoop::join(field, edge1, edge2);
                return;
            }
            _ => {
                field.grid_loop().inconsistent = true;
                return;
            }
        }

        if end1_vertex == end2_vertex {
            if status == Edge::Undecided {
                if field.grid_loop().decided_line != 0 {
                    GridLoop::decide_chain(field, edge1, Edge::Blank);
                    GridLoop::decide_chain(field, edge2, Edge::Blank);
                    GridLoop::check_chain_neighborhood(field, edge1);
                    GridLoop::check_chain_neighborhood(field, edge2);
                    return;
                }
            } else if status == Edge::Line {
                if field.grid_loop().decided_line != item1.chain_size + item2.chain_size {
                    field.grid_loop().inconsistent = true;
                    return;
                } else {
                    field.grid_loop().fully_solved = true;
                    GridLoop::has_fully_solved(field);
                }
            }
        }

        let grid_loop = field.grid_loop();

        let mut end1_item = grid_loop[end1_edge];
        let mut end2_item = grid_loop[end2_edge];

        // concatenate 2 lists
        mem::swap(&mut end1_item.chain_next, &mut end2_item.chain_next);

        // update chain_size
        let new_size = end1_item.chain_size + end2_item.chain_size;
        end1_item.chain_size = new_size;
        end2_item.chain_size = new_size;

        // update chain_end_points
        end1_item.chain_end_points = (end1_vertex, end2_vertex);
        end2_item.chain_end_points = (end1_vertex, end2_vertex);

        // update chain_another_end_edge
        end1_item.chain_another_end_edge = end2_edge;
        end2_item.chain_another_end_edge = end1_edge;

        grid_loop[end1_edge] = end1_item;
        grid_loop[end2_edge] = end2_item;

        grid_loop.queue.push(end1_vertex.0);
        grid_loop.queue.push(end2_vertex.0);
    }
    fn inspect_vertex<T: GridLoopField>(field: &mut T, pos: LP) {
        let mut line = FixVec::new();
        let mut undecided = FixVec::new();

        //for &(dy, dx) in [(1, 0), (0, 1), (-1, 0), (0, -1)].iter() {
        for &d in &FOUR_NEIGHBOURS {
            let pos_edge = pos + d;
            if field.grid_loop().is_valid_lp(pos_edge) {
                let id = field.grid_loop().grid.index_lp(pos_edge);
                let status = field.grid_loop().grid[id].edge_status;
                if status == Edge::Line {
                    line.push(EdgeId(id));
                } else if status == Edge::Undecided {
                    undecided.push(EdgeId(id));
                }
            }
        }

        if line.len() >= 3 {
            field.grid_loop().inconsistent = true;
            return;
        }

        if line.len() == 2 {
            for &e in &undecided {
                GridLoop::decide_edge_internal(field, e, Edge::Blank);
            }
            GridLoop::join(field, line[0], line[1]);
            return;
        }

        if line.len() == 1 {
            let eid = line[0];
            let vid = VtxId(field.grid_loop().grid.index_lp(pos));
            let line_size = field.grid_loop()[eid].chain_size;
            let another_end = field.grid_loop().another_end_id(vid, eid);

            // TODO: handle -1 / -2 properly
            let mut cand = -1;
            for &ud in &undecided {
                if field.grid_loop().is_end_of_chain(ud)
                    && field.grid_loop().is_end_of_chain_vertex(ud, vid)
                {
                    let ud_another_end = field.grid_loop().another_end_id(vid, ud);
                    if line_size == field.grid_loop().decided_line || another_end != ud_another_end
                    {
                        if cand == -1 {
                            cand = ud.0 as i32;
                        } else {
                            cand = -2;
                        }
                    } else {
                        GridLoop::decide_edge_internal(field, ud, Edge::Blank);
                        return;
                    }
                }
            }

            if cand == -1 {
                field.grid_loop().inconsistent = true;
            } else if cand != -2 {
                GridLoop::join(field, eid, EdgeId(cand as usize));
            }
        }

        if line.len() == 0 {
            if undecided.len() == 2 {
                GridLoop::join(field, undecided[0], undecided[1]);
            } else if undecided.len() == 1 {
                GridLoop::decide_edge_internal(field, undecided[0], Edge::Blank);
            }
        }
    }
}
pub trait GridLoopField {
    fn grid_loop(&mut self) -> &mut GridLoop;
    fn check_neighborhood(&mut self, pos: LP);
    fn inspect(&mut self, pos: LP);
}
impl GridLoopField for GridLoop {
    fn grid_loop(&mut self) -> &mut GridLoop {
        self
    }
    fn check_neighborhood(&mut self, pos: LP) {
        if pos.0 % 2 == 1 {
            GridLoop::check(self, pos + D(-1, 0));
            GridLoop::check(self, pos + D(1, 0));
        } else {
            GridLoop::check(self, pos + D(0, -1));
            GridLoop::check(self, pos + D(0, 1));
        }
    }
    fn inspect(&mut self, _: LP) {}
}
pub struct QueueActiveGridLoopField<'a, T: GridLoopField + 'a> {
    field: &'a mut T,
    finalize_required: bool,
}
impl<'a, T: GridLoopField> Deref for QueueActiveGridLoopField<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.field
    }
}
impl<'a, T: GridLoopField> DerefMut for QueueActiveGridLoopField<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.field
    }
}
impl<'a, T: GridLoopField> QueueActiveGridLoopField<'a, T> {
    fn new(field: &'a mut T) -> QueueActiveGridLoopField<'a, T> {
        if field.grid_loop().queue.is_started() {
            QueueActiveGridLoopField {
                field,
                finalize_required: false,
            }
        } else {
            field.grid_loop().queue.start();
            QueueActiveGridLoopField {
                field,
                finalize_required: true,
            }
        }
    }
}
impl<'a, T: GridLoopField> Drop for QueueActiveGridLoopField<'a, T> {
    fn drop(&mut self) {
        if self.finalize_required {
            GridLoop::queue_pop_all(self.field);
            self.field.grid_loop().queue.finish();
        }
    }
}

struct FixVec {
    data: [EdgeId; 4],
    idx: usize,
}
impl FixVec {
    fn new() -> FixVec {
        FixVec {
            data: [EdgeId(0); 4],
            idx: 0,
        }
    }
    fn push(&mut self, e: EdgeId) {
        let idx2 = self.idx;
        self.idx += 1;
        self.data[idx2] = e;
    }
    fn len(&self) -> usize {
        self.idx
    }
}
impl Index<usize> for FixVec {
    type Output = EdgeId;
    fn index(&self, index: usize) -> &EdgeId {
        &self.data[index]
    }
}
impl<'a> IntoIterator for &'a FixVec {
    type Item = &'a EdgeId;
    type IntoIter = ::std::slice::Iter<'a, EdgeId>;
    fn into_iter(self) -> Self::IntoIter {
        self.data[0..self.idx].into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_grid_loop_test(input: &[&str], expected: &[&str], inconsistent: bool) {
        let height = (input.len() / 2) as i32;
        let width = (input[0].len() / 2) as i32;
        let mut grid_loop = GridLoop::new(height, width);

        for y in 0..(input.len() as i32) {
            let mut row_iter = input[y as usize].chars();

            for x in 0..(input[0].len() as i32) {
                let ch = row_iter.next().unwrap();

                if !grid_loop.is_edge(LP(y, x)) {
                    continue;
                }
                match ch {
                    '|' | '-' => GridLoop::decide_edge(&mut grid_loop, LP(y, x), Edge::Line),
                    'x' => GridLoop::decide_edge(&mut grid_loop, LP(y, x), Edge::Blank),
                    _ => (),
                }
            }
        }

        let mut expected_decided_edge = 0;
        let mut expected_decided_line = 0;

        for y in 0..(input.len() as i32) {
            let mut row_iter = expected[y as usize].chars();

            for x in 0..(input[0].len() as i32) {
                let ch = row_iter.next().unwrap();

                if !grid_loop.is_edge(LP(y, x)) {
                    continue;
                }

                let expected_edge;
                match ch {
                    '|' | '-' => {
                        expected_decided_edge += 1;
                        expected_decided_line += 1;
                        expected_edge = Edge::Line;
                    }
                    'x' => {
                        expected_decided_edge += 1;
                        expected_edge = Edge::Blank;
                    }
                    _ => {
                        expected_edge = Edge::Undecided;
                    }
                }
                assert_eq!(
                    grid_loop.get_edge(LP(y, x)),
                    expected_edge,
                    "Comparing at y={}, x={}",
                    y,
                    x
                );
            }
        }

        assert_eq!(grid_loop.num_decided_edges(), expected_decided_edge);
        assert_eq!(grid_loop.num_decided_lines(), expected_decided_line);
        assert_eq!(grid_loop.inconsistent(), inconsistent);
    }

    #[test]
    fn test_corner() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+-+ + +",
                "      x",
                "+ + + +",
                "       ",
                "+ + + +",
                "      |",
                "+x+ + +",
            ],
            &[
                "+-+ +x+",
                "|     x",
                "+ + + +",
                "       ",
                "+ + + +",
                "x     |",
                "+x+ +-+",
            ],
            false,
        );
    }

    #[test]
    fn test_two_lines() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+ + + +",
                "       ",
                "+ +-+ +",
                "  |    ",
                "+ + + +",
                "       ",
                "+ + + +",
            ],
            &[
                "+ + + +",
                "  x    ",
                "+x+-+ +",
                "  |    ",
                "+ + + +",
                "       ",
                "+ + + +",
            ],
            false,
        );
    }

    #[test]
    fn test_joined_lines() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+ + + +",
                "  x    ",
                "+x+ + +",
                "x      ",
                "+ +x+ +",
                "  x    ",
                "+ +-+ +",
            ],
            &[
                "+x+x+ +",
                "x x    ",
                "+x+-+ +",
                "x |    ",
                "+-+x+ +",
                "| x    ",
                "+-+-+ +",
            ],
            false,
        );
    }

    #[test]
    fn test_line_close1() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+ + + +",
                "       ",
                "+ + +-+",
                "|   |  ",
                "+ + +-+",
                "       ",
                "+ + + +",
            ],
            &[
                "+ +-+-+",
                "    x |",
                "+ +x+-+",
                "|   | x",
                "+ +x+-+",
                "    x |",
                "+ +-+-+",
            ],
            false,
        );
    }

    #[test]
    fn test_line_close2() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+ + + +",
                "       ",
                "+ + +-+",
                "    |  ",
                "+ + +-+",
                "       ",
                "+ + + +",
            ],
            &[
                "+ + + +",
                "    x  ",
                "+ +x+-+",
                "    |  ",
                "+ +x+-+",
                "    x  ",
                "+ + + +",
            ],
            false,
        );
    }

    #[test]
    fn test_fully_solved() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        run_grid_loop_test(
            &[
                "+ + + +",
                "       ",
                "+ + +-+",
                "    | |",
                "+ + +-+",
                "       ",
                "+ + + +",
            ],
            &[
                "+x+x+x+",
                "x x x x",
                "+x+x+-+",
                "x x | |",
                "+x+x+-+",
                "x x x x",
                "+x+x+x+",
            ],
            false,
        );
    }

    #[test]
    fn test_inout_rule1() {
        let mut field = GridLoop::new(5, 5);
        GridLoop::decide_edge(&mut field, LP(3, 4), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(3, 6), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(4, 3), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(4, 7), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(6, 3), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(6, 7), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(7, 6), Edge::Line);

        GridLoop::apply_inout_rule(&mut field);

        assert_eq!(field.get_edge(LP(7, 4)), Edge::Line);
        assert_eq!(field.inconsistent(), false);
    }

    #[test]
    fn test_inout_rule2() {
        let mut field = GridLoop::new(5, 5);
        GridLoop::decide_edge(&mut field, LP(3, 4), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(3, 6), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(4, 3), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(4, 7), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(6, 3), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(6, 7), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(7, 4), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(7, 6), Edge::Line);

        GridLoop::apply_inout_rule(&mut field);

        assert_eq!(field.inconsistent(), true);
    }

    #[test]
    fn test_inout_rule3() {
        let mut field = GridLoop::new(5, 5);
        GridLoop::decide_edge(&mut field, LP(5, 0), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(5, 2), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(5, 4), Edge::Blank);
        GridLoop::decide_edge(&mut field, LP(4, 5), Edge::Line);
        GridLoop::decide_edge(&mut field, LP(2, 5), Edge::Line);

        GridLoop::apply_inout_rule(&mut field);

        assert_eq!(field.get_edge(LP(0, 5)), Edge::Line);
        assert_eq!(field.inconsistent(), false);
    }
}
