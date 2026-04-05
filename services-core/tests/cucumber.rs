mod steps;

use cucumber::World;
use steps::WhisperWorld;

fn main() {
    futures::executor::block_on(WhisperWorld::run("tests/features"));
}
