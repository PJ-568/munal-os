use crate::stats::SystemStats;
use crate::{network::TcpStack, time::SystemClock};
use applib::StyleSheet;
use rand::rngs::SmallRng;

pub struct System {
    pub clock: SystemClock,
    pub tcp_stack: TcpStack,
    pub rng: SmallRng,
    pub stylesheet: &'static StyleSheet,
    pub stats: SystemStats,
}
