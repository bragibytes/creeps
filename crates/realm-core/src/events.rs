use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::task::JoinHandle;
use tokio::time;

use crate::world::{next_mob_id, World};

pub struct WorldEventManager {
    world: World,
    notify: Arc<dyn Fn(String) + Send + Sync>,
    raid_active: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl WorldEventManager {
    pub fn new<F>(world: World, notify: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        Self {
            world,
            notify: Arc::new(notify),
            raid_active: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        let world = self.world.clone();
        let notify = Arc::clone(&self.notify);
        let raid_active = Arc::clone(&self.raid_active);

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5 * 60));
            loop {
                interval.tick().await;
                if raid_active.load(Ordering::Relaxed) {
                    continue;
                }
                if rand::thread_rng().gen::<f64>() > 0.35 {
                    continue;
                }
                start_goblin_raid(&world, &notify, &raid_active);
            }
        });

        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

fn start_goblin_raid(
    world: &World,
    notify: &Arc<dyn Fn(String) + Send + Sync>,
    raid_active: &Arc<AtomicBool>,
) {
    let tmpl_hp = world.mobs().get("goblin_scout").map(|m| m.hp);
    let Some(hp) = tmpl_hp else { return };

    world.get_room_mut("eldermoor_square", |room| {
        raid_active.store(true, Ordering::Relaxed);
        notify("*** WORLD EVENT: Goblins raid Eldermoor Town Square! ***".into());
        for _ in 0..3 {
            room.mobs.push(crate::world::LiveMob {
                instance_id: next_mob_id(),
                template_id: "goblin_scout".into(),
                hp,
                max_hp: hp,
                elite: None,
            });
        }
    });

    let notify = Arc::clone(notify);
    let raid_active = Arc::clone(raid_active);
    tokio::spawn(async move {
        time::sleep(Duration::from_secs(5 * 60)).await;
        raid_active.store(false, Ordering::Relaxed);
        notify("The goblin raid on Eldermoor has ended.".into());
    });
}