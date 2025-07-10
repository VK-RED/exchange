use std::time::Instant;

pub struct Observer {
    pub start_time: Instant,
    pub route: String,
}

impl Observer {
    pub fn new(start_time: Instant, route:String) -> Self {
        Self {
            start_time,
            route
        }
    }
}