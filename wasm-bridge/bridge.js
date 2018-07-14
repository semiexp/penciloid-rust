class PuzrsBridge {
    constructor(puzrs_wasm_path) {
        this.isReady = false;

        let env = {
            log: Math.log,
            cosf: Math.cos,
            cos: Math.cos,
            sinf: Math.sin,
            sin: Math.sin,
            exp: Math.exp
        };

        fetch(puzrs_wasm_path)
            .then(response => response.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, { env }))
            .then(results => {
                this.isReady = true;
                this.imported = results.instance.exports;
            });
    }

    generateNumberlink(constraints) {
        const height = constraints.height;
        const width = constraints.width;
        const seed1 = Math.random();
        const seed2 = Math.random();
        const problemAddress = this.imported.numberlink_generate(
            height,
            width,
            constraints.empty_width || 0,
            constraints.corner_clue ? constraints.corner_clue.low : -1,
            constraints.corner_clue ? constraints.corner_clue.high : -1,
            constraints.minimum_chain_length || 3,
            constraints.forbid_adjacent_clue,
            seed1,
            seed2
        );
        const problemData = new Uint8Array(this.imported.memory.buffer, problemAddress, height * width);

        let ret = [];
        for (let y = 0; y < height; ++y) {
            let row = [];
            for (let x = 0; x < width; ++x) {
                row.push(problemData[y * width + x]);
            }
            ret.push(row);
        }

        return {
            height,
            width,
            board: ret
        };
    }
}
