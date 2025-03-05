use std::collections::VecDeque;

// Statistics for the ping operation
pub struct PingStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub rtt_sum: f64,
    pub rtt_min: f64,
    pub rtt_max: f64,
    pub rtt_history: VecDeque<f64>,
}

impl PingStats {
    pub fn new() -> Self {
        PingStats {
            packets_sent: 0,
            packets_received: 0,
            rtt_sum: 0.0,
            rtt_min: f64::MAX,
            rtt_max: 0.0,
            rtt_history: VecDeque::with_capacity(10),
        }
    }

    pub fn update(&mut self, rtt: f64) {
        self.packets_received += 1;
        self.rtt_sum += rtt;
        self.rtt_min = self.rtt_min.min(rtt);
        self.rtt_max = self.rtt_max.max(rtt);
        
        // Add to history (used for moving average)
        self.rtt_history.push_back(rtt);
        if self.rtt_history.len() > 10 {
            self.rtt_history.pop_front();
        }
    }

    pub fn avg_rtt(&self) -> f64 {
        if self.packets_received == 0 {
            return 0.0;
        }
        self.rtt_sum / self.packets_received as f64
    }

    pub fn packet_loss(&self) -> f64 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        ((self.packets_sent - self.packets_received) as f64 / self.packets_sent as f64) * 100.0
    }
} 