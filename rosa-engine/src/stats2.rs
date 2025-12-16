use rosa_lib::mv::Mv;

pub struct SearchStats {
    nodes:u64,
    tt_hits:u64,
}

impl SearchStats {
    pub fn new() -> Self {
        SearchStats {
            nodes: 0,
            tt_hits: 0,
        }
    }

    pub fn node(&mut self) {
        self.nodes += 1;
    }

    pub fn tt_hit(&mut self) {
        self.tt_hits += 1;
    }
}

pub struct GlobalStats {
    pub nodes_per_depth: Vec<u64>,
    pub total_nodes: u64,

    pub start_time: std::time::Instant,
    pub time_per_depth: Vec<std::time::Instant>,
}

impl GlobalStats {
    pub fn new() -> Self {
        GlobalStats {
            nodes_per_depth: Vec::new(),
            total_nodes: 0,
            start_time: std::time::Instant::now(),
            time_per_depth: Vec::new(),
        }
    }

    pub fn depth_end(&mut self, stats: Vec<SearchStats>, pv: Mv, score: i32, depth: u8) {
        let mut nodes = 0;
        let mut tt_hits = 0;
        for stat in stats {
            nodes += stat.nodes;
            tt_hits += stat.tt_hits;
        }

        self.nodes_per_depth.push(nodes);
        self.total_nodes += nodes;
        
        let finish_time = std::time::Instant::now();
        self.time_per_depth.push(finish_time);

        println!( "info depth {} pv{} time {} score cp {} nodes {}, nps {}, tbhits {}" ,
            depth,
            pv,
            finish_time.duration_since(self.start_time).as_millis(),
            score,
            nodes,
            nodes / finish_time.duration_since(self.start_time).as_secs().max(1),
            tt_hits,
        );
    }

    pub fn search_end(&self) {

    }

}
