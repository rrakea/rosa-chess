use rosa_lib::mv::Mv;

pub struct Stats {
    pub nodes_current_depth: u64,
    pub nodes_per_depth: Vec<u64>,
    pub total_nodes: u64,

    pub start_time: std::time::Instant,
    pub time_per_depth: Vec<std::time::Instant>,

    pub tt_hits: u64,

    pub depth: u8,
}

const PRINT_DEBUG_INFO: bool = false;

impl Stats {
    pub fn new() -> Self {
        Stats {
            nodes_current_depth: 0,
            nodes_per_depth: Vec::new(),
            total_nodes: 0,
            start_time: std::time::Instant::now(),
            time_per_depth: Vec::new(),
            tt_hits: 0,
            depth: 1,
        }
    }

    pub fn node(&mut self) {
        self.nodes_current_depth += 1;
    }

    pub fn tt_hit(&mut self) {
        self.tt_hits += 1;
    }

    pub fn depth_done(&mut self) {
        self.nodes_per_depth.push(self.nodes_current_depth);
        self.total_nodes += self.nodes_current_depth;
        self.nodes_current_depth = 0;
        self.depth += 1;
        self.time_per_depth.push(std::time::Instant::now());
    }

    pub fn print_info(&self, pv: Mv, score: i32) {
        println!( "info depth {} pv{} time {} score cp {} nodes {}, nps {}, tbhits {}" ,
            self.depth,
            pv,
            self.time_per_depth.last().unwrap().elapsed().as_millis(),
            score,
            self.nodes_current_depth,
            self.total_nodes / self.start_time.elapsed().as_secs().max(1),
            self.tt_hits,
        );
    }

    pub fn search_done(&self, pv: Mv, score: i32) {
        println!("bestmove {}", pv);
        self.print_info(pv, score);
        if PRINT_DEBUG_INFO {
            self.print_debug_info();
        }
    }

    pub fn print_debug_info(&self) {}

}