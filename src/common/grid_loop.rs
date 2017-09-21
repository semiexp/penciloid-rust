use super::super::{Coord, Y, X, Grid, FiniteSearchQueue};

use std::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Undecided,
    Line,
    Blank,
}
#[derive(Clone, Copy)]
struct GridLoopItem {
    edge_status: Edge,
    chain_end_points: (usize, usize),
    chain_next: usize,
    chain_another_end_edge: usize,
    chain_size: i32,
}
pub struct GridLoop {
    grid: Grid<GridLoopItem>,
    inconsistent: bool,
    fully_solved: bool,
    decided_line: i32,
    decided_edge: i32,
    queue: FiniteSearchQueue,
}
impl GridLoop {
    pub fn new(height: i32, width: i32) -> GridLoop {
        let mut grid = Grid::new(height * 2 + 1, width * 2 + 1, GridLoopItem {
            edge_status: Edge::Undecided,
            chain_end_points: (0, 0),
            chain_next: 0,
            chain_another_end_edge: 0,
            chain_size: 0,
        });

        for y in 0..(height * 2 + 1) {
            for x in 0..(width * 2 + 1) {
                if y % 2 == x % 2 {
                    continue;
                }
                let id = grid.index((Y(y), X(x)));
                grid[(Y(y), X(x))] = GridLoopItem {
                    edge_status: Edge::Undecided,
                    chain_end_points: if y % 2 == 0 { (id - 1, id + 1) } else { (id - (width * 2 + 1) as usize, id + (width * 2 + 1) as usize) },
                    chain_next: id,
                    chain_another_end_edge: id,
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
            let edge1 = ret.grid.index((Y(0), X(1)));
            let edge2 = ret.grid.index((Y(1), X(0)));
            GridLoop::join(&mut ret, edge1, edge2);
        }
        {
            let edge1 = ret.grid.index((Y(0), X(width * 2 - 1)));
            let edge2 = ret.grid.index((Y(1), X(width * 2)));
            GridLoop::join(&mut ret, edge1, edge2);
        }
        {
            let edge1 = ret.grid.index((Y(height * 2 - 1), X(0)));
            let edge2 = ret.grid.index((Y(height * 2), X(1)));
            GridLoop::join(&mut ret, edge1, edge2);
        }
        {
            let edge1 = ret.grid.index((Y(height * 2 - 1), X(width * 2)));
            let edge2 = ret.grid.index((Y(height * 2), X(width * 2 - 1)));
            GridLoop::join(&mut ret, edge1, edge2);
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
    pub fn get_edge(&self, cd: Coord) -> Edge {
        self.grid[cd].edge_status
    }
    pub fn get_edge_safe(&self, cd: Coord) -> Edge {
        if self.is_valid_coord(cd) {
            self.get_edge(cd)
        } else {
            Edge::Blank
        }
    }
    pub fn is_valid_coord(&self, (Y(y), X(x)): Coord) -> bool {
        0 <= y && y < self.grid.height() && 0 <= x && x < self.grid.width()
    }
    pub fn is_vertex(&self, (Y(y), X(x)): Coord) -> bool {
        y % 2 == 0 && x % 2 == 0
    }
    pub fn is_edge(&self, (Y(y), X(x)): Coord) -> bool {
        y % 2 != x % 2
    }
    
    // public modifier
    pub fn decide_edge<T: GridLoopField>(field: &mut T, loc: Coord, status: Edge) {
        if !field.grid_loop().is_valid_coord(loc) {
            if status != Edge::Blank {
                field.grid_loop().inconsistent = true;
            }
            return;
        }

        let id = field.grid_loop().grid.index(loc);
        let current_status = field.grid_loop().grid[id].edge_status;

        if current_status == status {
            return;
        }
        if current_status != Edge::Undecided {
            field.grid_loop().inconsistent = true;
            return;
        }

        if field.grid_loop().queue.is_started() {
            GridLoop::decide_edge_internal(field, id, status);
        } else {
            field.grid_loop().queue.start();
            GridLoop::decide_edge_internal(field, id, status);
            GridLoop::queue_pop_all(field);
            field.grid_loop().queue.finish();
        }
    }
    pub fn check<T: GridLoopField>(field: &mut T, cd: Coord) {
        if !field.grid_loop().is_valid_coord(cd) {
            return;
        }

        let id = field.grid_loop().grid.index(cd);
        if field.grid_loop().queue.is_started() {
            field.grid_loop().queue.push(id);
        } else {
            field.grid_loop().queue.start();
            field.grid_loop().queue.push(id);
            GridLoop::queue_pop_all(field);
            field.grid_loop().queue.finish();
        }
    }

    // private accessor
    fn another_end_id(&self, origin: usize, edge: usize) -> usize {
        let edge_data = self.grid[edge];
        edge_data.chain_end_points.0 + edge_data.chain_end_points.1 - origin
    }
    fn is_end_of_chain(&self, id: usize) -> bool {
        let id2 = self.grid[id].chain_another_end_edge;
        self.grid[id2].chain_another_end_edge == id
    }
    fn is_end_of_chain_vertex(&self, edge: usize, vtx: usize) -> bool {
        let ends = self.grid[edge].chain_end_points;
        ends.0 == vtx || ends.1 == vtx
    }

    // private modifier
    fn queue_pop_all<T: GridLoopField>(field: &mut T) {
        while !field.grid_loop().queue.empty() {
            let id = field.grid_loop().queue.pop();
            let cd = field.grid_loop().grid.coord(id);
            field.inspect(cd);
            if field.grid_loop().is_vertex(cd) {
                GridLoop::inspect_vertex(field, cd);
            }
        }
    }
    fn decide_edge_internal<T: GridLoopField>(field: &mut T, id: usize, status: Edge) {
        let current_status = field.grid_loop().grid[id].edge_status;

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
    fn decide_chain<T: GridLoopField>(field: &mut T, edge: usize, status: Edge) {
        let gl = field.grid_loop();
        let sz = gl.grid[edge].chain_size;
        gl.decided_edge += sz;
        if status == Edge::Line {
            gl.decided_line += sz;
        }
        let mut pt = edge;
        loop {
            gl.grid[pt].edge_status = status;
            pt = gl.grid[pt].chain_next;
            if pt == edge {
                break;
            }
        }
    }
    fn check_chain_neighborhood<T: GridLoopField>(field: &mut T, edge: usize) {
        let mut pt = edge;
        loop {
            let cd = field.grid_loop().grid.coord(pt);
            field.check_neighborhood(cd);
            pt = field.grid_loop().grid[pt].chain_next;
            if pt == edge {
                break;
            }
        }
    }
    fn join<T: GridLoopField>(field: &mut T, edge1: usize, edge2: usize) {
        let mut item1 = field.grid_loop().grid[edge1];
        let mut item2 = field.grid_loop().grid[edge2];
        
        if !field.grid_loop().is_end_of_chain(edge1) || !field.grid_loop().is_end_of_chain(edge2) {
            return;
        }
        if item1.chain_another_end_edge == edge2 {
            return;
        }

        // ensure item1.0 == item2.0
        match (item1.chain_end_points, item2.chain_end_points) {
            ((ex, _), (ey, _)) if ex == ey => (),
            ((ex, _), (_, ey)) if ex == ey => mem::swap(&mut item2.chain_end_points.0, &mut item2.chain_end_points.1),
            ((_, ex), (ey, _)) if ex == ey => mem::swap(&mut item1.chain_end_points.0, &mut item1.chain_end_points.1),
            ((_, ex), (_, ey)) if ex == ey => {
                mem::swap(&mut item1.chain_end_points.0, &mut item1.chain_end_points.1);
                mem::swap(&mut item2.chain_end_points.0, &mut item2.chain_end_points.1);
            },
            _ => return
        }

        let origin = item1.chain_end_points.0;
        let end1_vertex = field.grid_loop().another_end_id(origin, edge1);
        let end2_vertex = field.grid_loop().another_end_id(origin, edge2);
        let end1_edge = field.grid_loop().grid[edge1].chain_another_end_edge;
        let end2_edge = field.grid_loop().grid[edge2].chain_another_end_edge;
        let status;

        match (field.grid_loop().grid[edge1].edge_status, field.grid_loop().grid[edge2].edge_status) {
            (status1, status2) if status1 == status2 => status = status1,
            (Edge::Undecided, status2) => {
                GridLoop::decide_chain(field, edge1, status2);
                GridLoop::check_chain_neighborhood(field, edge1);
                GridLoop::join(field, edge1, edge2);
                return;
            },
            (status1, Edge::Undecided) => {
                GridLoop::decide_chain(field, edge2, status1);
                GridLoop::check_chain_neighborhood(field, edge2);
                GridLoop::join(field, edge1, edge2);
                return;
            },
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
                    // GridLoop::has_fully_solved(field);
                }
            }
        }

        let grid_loop = field.grid_loop();

        let mut end1_item = grid_loop.grid[end1_edge];
        let mut end2_item = grid_loop.grid[end2_edge];

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

        grid_loop.grid[end1_edge] = end1_item;
        grid_loop.grid[end2_edge] = end2_item;

        grid_loop.queue.push(end1_vertex);
        grid_loop.queue.push(end2_vertex);
    }
    fn inspect_vertex<T: GridLoopField>(field: &mut T, (Y(y), X(x)): Coord) {
        let mut line = vec![];
        let mut undecided = vec![];

        for &(dy, dx) in [(1, 0), (0, 1), (-1, 0), (0, -1)].iter() {
            let cd = (Y(y + dy), X(x + dx));
            if field.grid_loop().is_valid_coord(cd) {
                let id = field.grid_loop().grid.index(cd);
                let status = field.grid_loop().grid[id].edge_status;
                if status == Edge::Line {
                    line.push(id);
                } else if status == Edge::Undecided {
                    undecided.push(id);
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
            let vid = field.grid_loop().grid.index((Y(y), X(x)));
            let line_size = field.grid_loop().grid[eid].chain_size;
            let another_end = field.grid_loop().another_end_id(vid, eid);

            let mut cand = -1;
            for &ud in &undecided {
                if field.grid_loop().is_end_of_chain(ud) && field.grid_loop().is_end_of_chain_vertex(ud, vid) {
                    let ud_another_end = field.grid_loop().another_end_id(vid, ud);
                    if line_size == field.grid_loop().decided_line || another_end != ud_another_end {
                        if cand == -1 {
                            cand = ud as i32;
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
                GridLoop::join(field, eid, cand as usize);
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
    fn check_neighborhood(&mut self, Coord);
    fn inspect(&mut self, Coord);
}
impl GridLoopField for GridLoop {
    fn grid_loop(&mut self) -> &mut GridLoop {
        self
    }
    fn check_neighborhood(&mut self, (Y(y), X(x)): Coord) {
        if y % 2 == 1 {
            GridLoop::check(self, (Y(y - 1), X(x)));
            GridLoop::check(self, (Y(y + 1), X(x)));
        } else {
            GridLoop::check(self, (Y(y), X(x - 1)));
            GridLoop::check(self, (Y(y), X(x + 1)));
        }
    }
    fn inspect(&mut self, (Y(y), X(x)): Coord) {
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

                if !grid_loop.is_edge((Y(y), X(x))) {
                    continue;
                }
                match ch {
                    '|' | '-' => GridLoop::decide_edge(&mut grid_loop, (Y(y), X(x)), Edge::Line),
                    'x' => GridLoop::decide_edge(&mut grid_loop, (Y(y), X(x)), Edge::Blank),
                    _ => (),
                }
            }
        }

        for y in 0..(input.len() as i32) {
            let mut row_iter = expected[y as usize].chars();

            for x in 0..(input[0].len() as i32) {
                let ch = row_iter.next().unwrap();

                if !grid_loop.is_edge((Y(y), X(x))) {
                    continue;
                }

                let expected_edge = match ch {
                    '|' | '-' => Edge::Line,
                    'x' => Edge::Blank,
                    _ => Edge::Undecided,
                };

                assert_eq!(grid_loop.get_edge((Y(y), X(x))), expected_edge, "Comparing at y={}, x={}", y, x);
            }
        }

        assert_eq!(grid_loop.inconsistent(), inconsistent);
    }

    #[test]
    fn test_corner() {
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
            false
        );
    }

    #[test]
    fn test_two_lines() {
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
            false
        );
    }

    #[test]
    fn test_joined_lines() {
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
            false
        );
    }

    #[test]
    fn test_line_close1() {
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
            false
        );
    }

    #[test]
    fn test_line_close2() {
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
            false
        );
    }
}
