use std::collections::HashSet;

use crate::geometry::{Rect, V2};
use crate::simulation::{ScatterPiece, SnowSimulation, Snowfall, Sprite, WindowSurface};

const DT: f32 = 1.0 / 60.0;
const AWAY: V2 = V2::new(-10_000.0, -10_000.0);

fn run(simulation: &mut SnowSimulation, seconds: f32, cursor: V2) {
    for _ in 0..(seconds / DT) as usize {
        simulation.step(DT, cursor);
    }
}

fn surface(id: isize, x: f32, top: f32, width: f32) -> WindowSurface {
    WindowSurface { id, frame: Rect { x, y: top - 300.0, width, height: 300.0 }, blockers: Vec::new(), corner_radius: 8.0, corner_exponent: 2.5 }
}

/// Snow area per layer, from the smoothed surfaces the renderer gets.
fn piles(simulation: &SnowSimulation) -> Vec<(V2, f32)> {
    let mut buffer = vec![0.0; simulation.column_count()];
    let layers = simulation.write_layers(&mut buffer);
    layers
        .iter()
        .map(|pile| {
            let (surface, bed) = buffer[pile.offset..pile.offset + 2 * pile.count].split_at(pile.count);
            (pile.origin, surface.iter().zip(bed).map(|(s, b)| s - b).sum::<f32>() * 2.0)
        })
        .collect()
}

fn assert_finite(simulation: &SnowSimulation) {
    let mut sprites = vec![Sprite::default(); simulation.sprite_count()];
    let count = simulation.write_sprites(&mut sprites);
    for sprite in &sprites[..count] {
        assert!(sprite.position.x.is_finite() && sprite.position.y.is_finite() && sprite.size.is_finite() && sprite.alpha.is_finite());
    }
    let mut buffer = vec![0.0; simulation.column_count()];
    simulation.write_layers(&mut buffer);
    assert!(buffer.iter().all(|h| h.is_finite()));
}

#[test]
fn snow_falls_and_settles_on_the_ground() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    run(&mut simulation, 1.0, AWAY);
    assert!(simulation.sprite_count() > 500, "flakes: {}", simulation.sprite_count());
    run(&mut simulation, 60.0, AWAY);
    assert!(simulation.has_snow());
    let piles = piles(&simulation);
    assert_eq!(piles.len(), 1);
    assert!(piles[0].1 > 1000.0, "ground area {}", piles[0].1);
    assert_finite(&simulation);
}

#[test]
fn off_stops_the_snowfall() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    run(&mut simulation, 30.0, AWAY);
    assert_eq!(simulation.sprite_count(), 0);
    assert!(!simulation.has_snow());
}

#[test]
fn cursor_sweeps_the_drift_aside() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    simulation.thaws = false;
    run(&mut simulation, 120.0, AWAY);
    simulation.set_snowfall(Snowfall::Off);
    run(&mut simulation, 15.0, AWAY);
    let before = piles(&simulation)[0].1;

    let mut sprayed = 0;
    for i in 0..120 {
        let x = 200.0 + i as f32 * 8.0;
        simulation.step(DT, V2::new(x, 10.0));
        sprayed = sprayed.max(simulation.sprite_count());
    }
    assert!(sprayed > 20, "no snow thrown up: {sprayed}");
    run(&mut simulation, 10.0, AWAY);
    let after = piles(&simulation)[0].1;
    // Swept snow lands again; it moves rather than vanishes.
    assert!((after - before).abs() / before < 0.1, "before {before}, after {after}");
    assert_finite(&simulation);
}

#[test]
fn a_gust_blows_all_the_snow_away() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    let none = HashSet::new();
    let window = [surface(7, 300.0, 600.0, 500.0)];
    simulation.update_windows(&window, 0.2, &none, &none, false);
    run(&mut simulation, 90.0, AWAY);
    assert_eq!(piles(&simulation).len(), 2);

    simulation.blow_away(1.0, false);
    run(&mut simulation, 1.5, AWAY);
    assert!(!simulation.has_snow(), "drifts left: {:?}", piles(&simulation));
    run(&mut simulation, 3.0, AWAY);
    // The new snowfall starts above the screen and hasn't reached the ground yet.
    assert!(!simulation.has_snow());
    let mut sprites = vec![Sprite::default(); 20_000];
    let count = simulation.write_sprites(&mut sprites);
    assert!(sprites[..count].iter().all(|sprite| sprite.position.y > 400.0), "snow left in the lower half");
    run(&mut simulation, 60.0, AWAY);
    assert!(simulation.has_snow(), "the snowfall didn't come back");
    assert_finite(&simulation);
}

#[test]
fn a_calm_gust_melts_the_drifts_where_they_lie() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    run(&mut simulation, 90.0, AWAY);
    simulation.set_snowfall(Snowfall::Off);
    let before = piles(&simulation)[0].1;
    let falling = simulation.sprite_count();
    simulation.blow_away(-1.0, true);
    simulation.step(DT, AWAY);
    assert!(simulation.sprite_count() <= falling, "a calm gust shouldn't throw anything up");
    run(&mut simulation, 2.0, AWAY);
    // Only the last of the snowfall, still landing, is left.
    let after = piles(&simulation).first().map_or(0.0, |p| p.1);
    assert!(after < before * 0.05, "before {before}, after {after}");
}

#[test]
fn snow_settles_on_a_window_and_drops_when_it_closes() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    let none = HashSet::new();
    let window = [surface(7, 300.0, 600.0, 500.0)];
    simulation.update_windows(&window, 0.2, &none, &none, false);
    run(&mut simulation, 90.0, AWAY);
    let layers = piles(&simulation);
    assert_eq!(layers.len(), 2, "no snow on the window");
    let ledge = layers.iter().find(|(origin, _)| origin.y > 500.0).expect("ledge");
    assert!(ledge.1 > 200.0, "ledge area {}", ledge.1);

    simulation.set_snowfall(Snowfall::Off);
    simulation.update_windows(&[], 0.2, &none, &none, false);
    simulation.step(DT, AWAY);
    assert!(simulation.sprite_count() > 20, "the window's snow didn't fall");
    run(&mut simulation, 5.0, AWAY);
    assert_eq!(piles(&simulation).len(), 1);
    assert_finite(&simulation);
}

#[test]
fn dragged_window_sheds_its_snow() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    simulation.thaws = false;
    let none = HashSet::new();
    simulation.update_windows(&[surface(3, 300.0, 600.0, 500.0)], 0.2, &none, &none, false);
    run(&mut simulation, 90.0, AWAY);
    simulation.set_snowfall(Snowfall::Off);
    run(&mut simulation, 10.0, AWAY);
    let resting = piles(&simulation).iter().find(|(origin, _)| origin.y > 500.0).map(|p| p.1).unwrap();

    // Shaken hard from side to side, as when a window is dragged back and forth.
    shake(&mut simulation, 3);
    run(&mut simulation, 3.0, AWAY);
    let shaken = piles(&simulation).iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1);
    assert!(shaken < resting * 0.7, "resting {resting}, after shaking {shaken}");
    assert_finite(&simulation);
}

#[test]
fn scattered_window_starts_the_held_snowfall() {
    let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Steady);
    simulation.set_snowfall_held(true);
    run(&mut simulation, 2.0, AWAY);
    assert_eq!(simulation.sprite_count(), 0, "snowfall should wait for the welcome window");

    let pieces: Vec<ScatterPiece> = (0..2000)
        .map(|i| ScatterPiece {
            position: V2::new(420.0 + (i % 70) as f32 * 6.0, 300.0 + (i / 70) as f32 * 6.0),
            velocity: V2::new(0.0, 60.0),
            color: [0.1, 0.09, 0.07],
        })
        .collect();
    simulation.scatter(&pieces);
    let mut released = false;
    for _ in 0..(10.0 / DT) as usize {
        simulation.step(DT, AWAY);
        released |= simulation.take_snowfall_released();
    }
    assert!(released, "the scattered snow never reached the ground");
    run(&mut simulation, 10.0, AWAY);
    assert!(simulation.sprite_count() > 200, "snowfall didn't start: {}", simulation.sprite_count());
    assert_finite(&simulation);
}

#[test]
fn a_window_slid_past_the_screen_edge_leaves_its_snow_heaped_against_it() {
    for (out, wall) in [(-300.0, 0.0), (1080.0, 1280.0)] {
        let mut simulation = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
        simulation.thaws = false;
        simulation.walls = (true, true);
        let none = HashSet::new();
        simulation.update_windows(&[surface(5, 400.0, 600.0, 500.0)], 0.2, &none, &none, false);
        run(&mut simulation, 90.0, AWAY);
        simulation.set_snowfall(Snowfall::Off);
        run(&mut simulation, 10.0, AWAY);
        let resting = ledge(&simulation);

        for step in 1..=180 {
            let x = 400.0 + (out - 400.0) * step as f32 / 180.0;
            simulation.update_windows(&[surface(5, x, 600.0, 500.0)], DT, &none, &none, false);
            simulation.step(DT, AWAY);
        }
        run(&mut simulation, 5.0, AWAY);
        let heaped = ledge(&simulation);
        // Three fifths of the window went past the wall, and their snow packs down once, to 0.6 of its bulk.
        assert!((heaped / resting - 0.76).abs() < 0.08, "resting {resting}, against the wall {heaped}");

        let mut buffer = vec![0.0; simulation.column_count()];
        let layers = simulation.write_layers(&mut buffer);
        let pile = layers.iter().find(|pile| pile.origin.y > 500.0).expect("ledge");
        let (surface, bed) = buffer[pile.offset..pile.offset + 2 * pile.count].split_at(pile.count);
        let depth = |i: usize| surface[i] - bed[i];
        let x = |i: usize| pile.origin.x + (i as f32 + 0.5) * 2.0;
        let past = (0..pile.count).filter(|&i| (x(i) - wall).abs() > 4.0 && (x(i) < 0.0 || x(i) > 1280.0)).map(depth).fold(0.0, f32::max);
        assert!(past < 0.01, "snow left past the edge at {wall}: {past}");
        let deepest = (0..pile.count).max_by(|&a, &b| depth(a).total_cmp(&depth(b))).unwrap();
        assert!((x(deepest) - wall).abs() < 30.0, "deepest snow at {}, the wall at {wall}", x(deepest));
        assert_finite(&simulation);
    }
}

fn shake(simulation: &mut SnowSimulation, id: isize) {
    let none = HashSet::new();
    let mut t = 0.0;
    for _ in 0..120 {
        t += DT;
        let x = 300.0 + 150.0 * (2.0 * std::f32::consts::PI * 4.0 * t).sin();
        simulation.update_windows(&[surface(id, x, 600.0, 500.0)], DT, &none, &none, false);
        simulation.step(DT, AWAY);
    }
}

fn ledge(simulation: &SnowSimulation) -> f32 {
    piles(simulation).iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1)
}

/// The same scenarios as the Swift harness, to compare the port against the original.
#[test]
#[ignore]
fn compare_with_swift() {
    let none = HashSet::new();
    let runs = 6;
    let (mut ground, mut ledge_area, mut dropped, mut shake_ratio, mut sweep_peak, mut sweep_ratio) = (0.0, 0.0, 0, 0.0, 0, 0.0);
    for _ in 0..runs {
        let mut a = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
        a.thaws = true;
        run(&mut a, 60.0, AWAY);
        ground += piles(&a)[0].1;

        let mut b = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
        b.update_windows(&[surface(3, 300.0, 600.0, 500.0)], 0.2, &none, &none, false);
        run(&mut b, 90.0, AWAY);
        b.set_snowfall(Snowfall::Off);
        run(&mut b, 10.0, AWAY);
        let resting = ledge(&b);
        ledge_area += resting;
        shake(&mut b, 3);
        run(&mut b, 3.0, AWAY);
        shake_ratio += ledge(&b) / resting;
        b.update_windows(&[surface(3, 300.0, 600.0, 500.0)], DT, &none, &none, false);
        run(&mut b, 3.0, AWAY);
        b.update_windows(&[], 0.2, &none, &none, false);
        b.step(DT, AWAY);
        dropped += b.sprite_count();

        let mut c = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
        run(&mut c, 120.0, AWAY);
        c.set_snowfall(Snowfall::Off);
        run(&mut c, 15.0, AWAY);
        let before = piles(&c)[0].1;
        let mut peak = 0;
        for i in 0..120 {
            c.step(DT, V2::new(200.0 + i as f32 * 8.0, 10.0));
            peak = peak.max(c.sprite_count());
        }
        run(&mut c, 10.0, AWAY);
        sweep_peak += peak;
        sweep_ratio += piles(&c)[0].1 / before;
    }
    let n = runs as f32;
    println!(
        "ground60 {:.0}  ledge90 {:.0}  shakeLeft {:.2}  dropped {}  sweepPeak {}  sweepKept {:.3}",
        ground / n,
        ledge_area / n,
        shake_ratio / n,
        dropped / runs,
        sweep_peak / runs,
        sweep_ratio / n
    );
}

#[test]
fn survives_a_long_chaotic_session() {
    let mut simulation = SnowSimulation::new(1920.0, 1080.0, Snowfall::Variable);
    let none = HashSet::new();
    let mut seed = 12345u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 40) as f32 / (1u32 << 24) as f32
    };
    let mut windows: Vec<(isize, f32, f32, f32)> = Vec::new();
    let mut cursor = V2::new(500.0, 100.0);
    for frame in 0..40_000 {
        if frame % 30 == 0 {
            let roll = next();
            if roll < 0.2 && windows.len() < 6 {
                windows.push((frame as isize + 1, next() * 1600.0 - 100.0, 200.0 + next() * 850.0, 60.0 + next() * 900.0));
            } else if roll < 0.3 && !windows.is_empty() {
                windows.remove((next() * windows.len() as f32) as usize % windows.len());
            }
        }
        for window in &mut windows {
            if next() < 0.05 {
                window.1 += (next() - 0.5) * 400.0;
                window.2 += (next() - 0.5) * 200.0;
            }
            if next() < 0.01 {
                window.3 = 20.0 + next() * 1200.0;
            }
        }
        let surfaces: Vec<WindowSurface> = windows.iter().map(|&(id, x, top, width)| surface(id, x, top, width)).collect();
        simulation.update_windows(&surfaces, DT, &none, &none, false);
        if next() < 0.3 {
            cursor = cursor + V2::new((next() - 0.5) * 300.0, (next() - 0.5) * 200.0);
        }
        if frame % 5000 == 0 {
            simulation.thaws = !simulation.thaws;
            simulation.accumulates = frame % 15000 != 0;
        }
        if frame == 20_000 {
            simulation.resize(1280.0, 720.0);
        }
        let dt = if next() < 0.01 { 0.05 } else { DT };
        simulation.step(dt, cursor);
    }
    assert_finite(&simulation);
}

#[test]
#[ignore]
fn step_cost() {
    let none = HashSet::new();
    let mut simulation = SnowSimulation::new(1920.0, 1080.0, Snowfall::Steady);
    simulation.thaws = true;
    let window = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 300.0, width: 900.0, height: 500.0 },
        blockers: Vec::new(),
        corner_radius: 8.0,
        corner_exponent: 2.5,
    }];
    simulation.update_windows(&window, 0.2, &none, &none, false);
    for _ in 0..60 * 30 {
        simulation.step(1.0 / 30.0, AWAY);
    }
    let steps = 30 * 120;
    let start = std::time::Instant::now();
    for i in 0..steps {
        simulation.step(1.0 / 30.0, V2::new((i % 1900) as f32, 20.0));
    }
    let us = start.elapsed().as_secs_f64() * 1e6 / steps as f64;
    println!("rust: {us:.1} µs per step, {} sprites", simulation.sprite_count());
}

#[test]
fn static_wind_affects_flake_motion() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Steady);
    sim.static_wind = 60.0;
    run(&mut sim, 3.0, AWAY);
    let mut sprites = vec![Sprite::default(); sim.sprite_count()];
    let count = sim.write_sprites(&mut sprites);
    assert!(count > 0);
    let avg_vx = sprites[..count].iter().map(|s| s.velocity.x).sum::<f32>() / count as f32;
    assert!(avg_vx > 10.0, "average horizontal velocity should be positive: {avg_vx}");
}

#[test]
fn window_leak_allows_snow_to_reach_ground() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.window_leak = 1.0;
    let win = [WindowSurface {
        id: 1,
        frame: Rect { x: 0.0, y: 100.0, width: 1280.0, height: 600.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win, 0.2, &none, &none, false);
    run(&mut sim, 40.0, AWAY);
    let p = piles(&sim);
    // With leak=1.0, all snow bypasses the window and falls to ground.
    // Window has 0 snow so it's omitted from piles (top <= 0.05).
    assert_eq!(p.len(), 1, "only ground should have snow");
    let ground_snow = p[0].1;
    assert!(ground_snow > 500.0, "ground snow should accumulate with leak=1.0: {ground_snow}");

    // Now test with leak=0.0: window catches snow
    let mut sim2 = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim2.window_leak = 0.0;
    sim2.update_windows(&win, 0.2, &none, &none, false);
    run(&mut sim2, 40.0, AWAY);
    let window_snow = piles(&sim2).iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1);
    assert!(window_snow > 500.0, "window should catch snow with leak=0.0: {window_snow}");
}

#[test]
fn motion_shed_releases_snow_on_movement() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.motion_shed = 2.0;

    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 100.0, width: 600.0, height: 400.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.2, &none, &none, false);
    run(&mut sim, 40.0, AWAY);

    let initial_window_snow = piles(&sim).iter().find(|(origin, _)| origin.y > 400.0).map_or(0.0, |p| p.1);
    assert!(initial_window_snow > 100.0, "initial snow must accumulate: {initial_window_snow}");

    // Move window rapidly by 300px in 16ms (speed ~ 18750 px/s)
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 500.0, y: 100.0, width: 600.0, height: 400.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    let moved_window_snow = piles(&sim).iter().find(|(origin, _)| origin.y > 400.0).map_or(0.0, |p| p.1);
    assert!(moved_window_snow < initial_window_snow, "snow should shed on fast movement: before={initial_window_snow}, after={moved_window_snow}");
}

#[test]
fn cascading_window_leak_distributes_snow_across_multiple_windows() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.window_leak = 0.5;

    // Two stacked windows: Upper window (y=500..700), Lower window (y=200..400)
    let wins = [
        WindowSurface {
            id: 1,
            frame: Rect { x: 100.0, y: 500.0, width: 800.0, height: 200.0 },
            blockers: Vec::new(),
            corner_radius: 0.0,
            corner_exponent: 2.0,
        },
        WindowSurface {
            id: 2,
            frame: Rect { x: 100.0, y: 200.0, width: 800.0, height: 200.0 },
            blockers: Vec::new(),
            corner_radius: 0.0,
            corner_exponent: 2.0,
        },
    ];
    sim.update_windows(&wins, 0.2, &none, &none, false);
    run(&mut sim, 40.0, AWAY);

    let all_piles = piles(&sim);
    let upper_snow = all_piles.iter().find(|(origin, _)| origin.y > 600.0).map_or(0.0, |p| p.1);
    let lower_snow = all_piles.iter().find(|(origin, _)| origin.y > 300.0 && origin.y < 600.0).map_or(0.0, |p| p.1);
    let ground_snow = all_piles.iter().find(|(origin, _)| origin.y < 50.0).map_or(0.0, |p| p.1);

    assert!(upper_snow > 50.0, "upper window should catch some snow: {upper_snow}");
    assert!(lower_snow > 50.0, "lower window should catch cascading snow that skipped upper: {lower_snow}");
    assert!(ground_snow > 50.0, "ground should receive cascading snow: {ground_snow}");
}

#[test]
fn cursor_force_pushes_snowflakes() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.cursor_force = 3.0;

    // Run a bit with cursor in center moving rightwards at high speed
    for i in 0..60 {
        sim.step(1.0 / 60.0, V2::new(400.0 + (i as f32) * 5.0, 400.0));
    }
    assert!(sim.sprite_count() > 0);
}

#[test]
fn stationary_cursor_does_not_push_snowflakes() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.cursor_force = 3.0;
    sim.static_wind = 0.0;
    run(&mut sim, 20.0, AWAY);

    let stationary_cursor = V2::new(600.0, 400.0);
    // Hold cursor stationary for multiple steps
    for _ in 0..15 {
        sim.step(1.0 / 60.0, stationary_cursor);
    }

    let mut sprites = vec![Sprite::default(); 5000];
    let count = sim.write_sprites(&mut sprites);
    // Flakes near the stationary cursor should NOT be shoved with huge velocities (> 80 px/s)
    let blown_flakes = sprites[..count].iter().filter(|s| {
        (s.position - stationary_cursor).length() < 70.0 && s.velocity.x.abs() > 80.0
    }).count();

    assert_eq!(blown_flakes, 0, "stationary cursor should not blow snowflakes away");
}

#[test]
fn snowflakes_wrap_around_edges() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.spawn_flake_for_test(V2::new(1309.0, 400.0), V2::new(100.0, -10.0), true);

    let mut sprites = vec![Sprite::default(); 100];
    sim.step(0.1, AWAY);
    let count = sim.write_sprites(&mut sprites);
    assert_eq!(count, 1, "snowflake should not be deleted, but wrapped around");
    assert!(
        sprites[0].position.x < 100.0,
        "snowflake should wrap from right to left side: {}",
        sprites[0].position.x
    );

    // Effect pieces (e.g. scatter) should NOT wrap around and should exit screen
    let mut sim_scatter = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim_scatter.scatter(&[ScatterPiece {
        position: V2::new(1309.0, 400.0),
        velocity: V2::new(100.0, -10.0),
        color: [1.0, 1.0, 1.0],
    }]);
    sim_scatter.step(0.1, AWAY);
    let count_scatter = sim_scatter.write_sprites(&mut sprites);
    assert_eq!(count_scatter, 0, "effect piece should not wrap around and should exit screen");
}

#[test]
fn window_movement_plows_snow_on_ground() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);

    // Accumulate snow on ground for 30s
    run(&mut sim, 30.0, AWAY);
    let p = piles(&sim);
    assert_eq!(p.len(), 1);
    assert!(p[0].1 > 500.0, "ground snow should accumulate");

    // Spawn window at x=300..700, touching ground drift at y=5.0
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 300.0, y: 5.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.2, &none, &none, false);

    // Move window across to x=500..900, plowing into ground snow
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 500.0, y: 5.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    // Verify that ground snow inside the window's footprint (e.g. x=600) was plowed down to ~5.0
    let mut heights = vec![0.0f32; sim.column_count()];
    let _layers = sim.write_layers(&mut heights);
    // Ground column at x=600:
    let col_600 = (600.0 / crate::simulation::COLUMN_WIDTH) as usize;
    let h_600 = heights[col_600];
    assert!(
        h_600 <= 6.0,
        "ground snow at x=600 under window should be carved down to ~5.0: got {h_600}"
    );
}

#[test]
fn workspace_switch_teleports_snowflakes() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);

    // Let flakes spawn
    for _ in 0..10 {
        sim.step(1.0 / 30.0, AWAY);
    }
    assert!(sim.sprite_count() > 0);

    // Trigger workspace switch to the right
    sim.sweep_window_ledges(1.0, 1.0, &none);
    assert!(sim.workspace_transition_until > 0.0);

    // Run steps during workspace transition
    let mut sprites = vec![Sprite::default(); 5000];
    for _ in 0..15 {
        sim.step(1.0 / 30.0, AWAY);
        let count = sim.write_sprites(&mut sprites);
        for s in &sprites[..count] {
            // During workspace transition, flakes wrap within [-35.0, 1280.0 + 35.0]
            assert!(
                s.position.x >= -45.0 && s.position.x <= 1280.0 + 45.0,
                "flake escaped wrapping bounds during workspace transition: {}",
                s.position.x
            );
        }
    }
}

#[test]
fn moving_window_creates_air_wake_on_snowflakes() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.static_wind = 0.0;

    // Spawn window
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 300.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.2, &none, &none, false);

    run(&mut sim, 20.0, AWAY);

    // Move window rapidly to the right (x: 300 -> 600)
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 600.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    // Verify that flakes in/near the window's wake have positive horizontal velocity
    let mut sprites = vec![Sprite::default(); 5000];
    let count = sim.write_sprites(&mut sprites);
    let pushed_flakes = sprites[..count].iter().filter(|s| {
        s.position.x >= 550.0 && s.position.x <= 1050.0
            && s.position.y >= 250.0 && s.position.y <= 650.0
            && s.velocity.x > 50.0
    }).count();

    assert!(pushed_flakes > 0, "moving window should blow snowflakes in its wake");
}

#[test]
fn workspace_switch_partially_sweeps_snowdrifts() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.thaws = false;
    sim.static_wind = 0.0;

    let win = [WindowSurface {
        id: 1,
        frame: Rect { x: 300.0, y: 400.0, width: 400.0, height: 200.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win, 0.2, &none, &none, false);

    // Accumulate snow
    run(&mut sim, 30.0, AWAY);
    sim.set_snowfall(Snowfall::Off);
    run(&mut sim, 10.0, AWAY);

    let all_piles_before = piles(&sim);
    let ground_before = all_piles_before[0].1;
    let ledge_before = all_piles_before.iter().find(|(origin, _)| origin.y > 300.0).unwrap().1;

    assert!(ground_before > 100.0, "ground should have accumulated snow");
    assert!(ledge_before > 50.0, "window ledge should have accumulated snow");

    // Perform workspace sweep
    sim.sweep_window_ledges(1.0, 1.0, &none);

    let all_piles_after = piles(&sim);
    let ground_after = all_piles_after[0].1;
    let ledge_after = all_piles_after.iter().find(|(origin, _)| origin.y > 300.0).map_or(0.0, |p| p.1);

    // Window ledges: all snow flies off into clouds and grains (no drifts remain on window!)
    assert_eq!(ledge_after, 0.0, "window ledge snow should fly away completely upon workspace sweep");

    // Ground: snowdrifts are partially swept, retaining most of their accumulation
    assert!(ground_after < ground_before, "ground snow should decrease slightly after sweep");
    assert!(ground_after > ground_before * 0.6, "ground snow should retain most of its drift: before={}, after={}", ground_before, ground_after);

    // Verify grains and cloud motes were spawned into the air
    assert!(sim.sprite_count() > 0, "snow sweep should launch flying grains and cloud motes");
}

#[test]
fn cursor_movement_creates_cloud_motes() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.thaws = false;
    sim.cursor_force = 1.0;

    // Place a snowdrift on the ground
    for i in 100..200 {
        sim.layers_mut()[0].set(i, 20.0);
    }
    sim.layers_mut()[0].refresh_top();

    // Move cursor rapidly across the snowdrift
    let dt = 1.0 / 60.0;
    sim.step(dt, V2::new(200.0, 10.0));
    sim.step(dt, V2::new(300.0, 10.0));
    sim.step(dt, V2::new(400.0, 10.0));

    // Verify that motes were generated
    let mut sprites = vec![Sprite::default(); 5000];
    let count = sim.write_sprites(&mut sprites);
    // Motes have soft == 0.98 in write_sprites
    let cloud_motes = sprites[..count].iter().filter(|s| s.soft > 0.95).count();
    assert!(cloud_motes > 0, "moving cursor through snow should generate cloud motes");
}

#[test]
fn constant_strong_wind_erodes_snowdrifts() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.thaws = false;

    // Manually place a tall snowdrift on the ground
    for i in 100..200 {
        sim.layers_mut()[0].set(i, 25.0);
    }
    sim.layers_mut()[0].refresh_top();
    let before = piles(&sim)[0].1;
    assert!(before > 0.0);

    // With calm wind (15 px/s), drift should remain stable (no erosion)
    sim.static_wind = 15.0;
    for _ in 0..60 {
        sim.step(1.0 / 30.0, AWAY);
    }
    let after_calm = piles(&sim)[0].1;
    assert!((after_calm - before).abs() < 0.1, "calm wind should not erode snowdrifts: before={}, after={}", before, after_calm);

    // With strong constant wind (75 px/s), drift should be eroded and shed powder
    sim.static_wind = 75.0;
    for _ in 0..120 {
        sim.step(1.0 / 30.0, AWAY);
    }
    let after_strong = piles(&sim)[0].1;
    assert!(after_strong < after_calm, "strong constant wind should erode snowdrift: calm={}, strong={}", after_calm, after_strong);
}

#[test]
fn snow_passes_through_windows_when_not_solid() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.thaws = false;
    sim.windows_solid = false; // lock screen mode

    let win = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 300.0, width: 800.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win, 0.016, &none, &none, false);

    // Run simulation for 40 seconds of heavy snowfall
    run(&mut sim, 40.0, AWAY);

    let all_piles = piles(&sim);
    let window_snow = all_piles.iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1);
    assert_eq!(window_snow, 0.0, "snow must not settle on window when windows_solid is false");

    let ground_snow = all_piles.iter().find(|(origin, _)| origin.y < 50.0).map_or(0.0, |p| p.1);
    assert!(ground_snow > 50.0, "snow must pass through window and settle on the ground: {ground_snow}");
}

#[test]
fn window_plow_emits_physical_grains_and_clouds() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.thaws = false;

    // Place a snowdrift on the ground (y=0..20)
    for i in 100..200 {
        sim.layers_mut()[0].set(i, 20.0);
    }
    sim.layers_mut()[0].refresh_top();

    // Spawn window touching ground snow (y=5.0)
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 5.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.016, &none, &none, false);

    // Rapidly move the window horizontally to plow ground snow
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 350.0, y: 5.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    let mut sprites = vec![Sprite::default(); 5000];
    let count = sim.write_sprites(&mut sprites);

    // Physical grains have soft <= 0.35 and alpha > 0.0
    let grains_count = sprites[..count].iter().filter(|s| (s.soft - 0.3).abs() < 0.05 || (s.soft - 0.2).abs() < 0.05).count();
    // Cloud motes have soft == 1.0
    let motes_count = sprites[..count].iter().filter(|s| s.soft > 0.95).count();

    assert!(grains_count > 0, "moving window plowing snow must emit physical grains: {grains_count}");
    assert!(motes_count > 0, "moving window plowing snow must emit cloud motes: {motes_count}");
}

#[test]
fn window_downward_crush_sprays_snow_up_and_outward() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.thaws = false;

    // Place a snowdrift on the ground (height = 30) spanning the full window width (cols 100..300 = x: 200..600)
    for i in 100..300 {
        sim.layers_mut()[0].set(i, 30.0);
    }
    sim.layers_mut()[0].refresh_top();

    // Window hovers just at the top of the drift (y = 30.0)
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 30.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.016, &none, &none, false);

    // Rapidly press the window DOWN into the drift to y = 5.0 (downward velocity = -1560 px/s)
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 5.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    let mut sprites = vec![Sprite::default(); 5000];
    let count = sim.write_sprites(&mut sprites);

    // Verify upward and outward spray of grains
    let upward_grains = sprites[..count].iter().filter(|s| {
        ((s.soft - 0.3).abs() < 0.05 || (s.soft - 0.2).abs() < 0.05) && s.velocity.y > 100.0
    }).count();

    let left_spray = sprites[..count].iter().filter(|s| {
        ((s.soft - 0.3).abs() < 0.05 || (s.soft - 0.2).abs() < 0.05) && s.velocity.x < -40.0
    }).count();

    let right_spray = sprites[..count].iter().filter(|s| {
        ((s.soft - 0.3).abs() < 0.05 || (s.soft - 0.2).abs() < 0.05) && s.velocity.x > 40.0
    }).count();

    assert!(upward_grains > 10, "downward crush must spray grains upward: {upward_grains}");
    assert!(left_spray > 3, "downward crush must spray grains to the left: {left_spray}");
    assert!(right_spray > 3, "downward crush must spray grains to the right: {right_spray}");
}

#[test]
fn test_lockscreen_stashes_desktop_windows_and_collects_on_lock_card() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.thaws = false;

    // 1. Regular desktop window accumulates snow
    let desktop_win = [WindowSurface {
        id: 101,
        frame: Rect { x: 100.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&desktop_win, 0.016, &none, &none, false);
    run(&mut sim, 30.0, AWAY);

    let desktop_snow = piles(&sim).iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1);
    assert!(desktop_snow > 10.0, "desktop window should have accumulated snow: {desktop_snow}");

    // 2. Lock screen activates: desktop window becomes elsewhere (dormant), lock card appears
    let mut elsewhere = HashSet::new();
    elsewhere.insert(101);

    let lock_card = [WindowSurface {
        id: 0x710c_5c23,
        frame: Rect { x: 440.0, y: 250.0, width: 400.0, height: 260.0 },
        blockers: Vec::new(),
        corner_radius: 16.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&lock_card, 0.016, &none, &elsewhere, false);

    // Desktop window snow must NOT be rendered / must not hang in air
    let active_piles = piles(&sim);
    let mid_air_desktop_snow = active_piles.iter().find(|(origin, _)| origin.x < 300.0 && origin.y > 500.0).map_or(0.0, |p| p.1);
    assert_eq!(mid_air_desktop_snow, 0.0, "dormant desktop window snow must not hang in air during lock");

    // Snow falls onto lock card
    run(&mut sim, 30.0, AWAY);
    let lock_snow = piles(&sim).iter().find(|(origin, _)| origin.x > 400.0 && origin.y > 450.0).map_or(0.0, |p| p.1);
    assert!(lock_snow > 10.0, "snow must land and accumulate on lock card: {lock_snow}");

    // 3. Screen unlocks: lock card gone, desktop window restored from dormant
    let empty_elsewhere = HashSet::new();
    sim.update_windows(&desktop_win, 0.016, &none, &empty_elsewhere, false);
    let restored_desktop_snow = piles(&sim).iter().find(|(origin, _)| origin.y > 500.0).map_or(0.0, |p| p.1);
    assert!(restored_desktop_snow > 10.0, "desktop snow should be restored after unlocking: {restored_desktop_snow}");
}

#[test]
fn test_window_wake_slider_controls_aerodynamic_cushion_and_does_not_disturb_ground_snow() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    // Put snow on ground
    for col in 100..300 {
        sim.layers_mut()[0].set(col, 25.0);
    }
    sim.layers_mut()[0].refresh_top();
    let initial_ground_snow = piles(&sim)[0].1;

    // Spawn window high above ground (ground y is 0, window y is 300..600)
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 300.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.2, &none, &none, false);

    // Set wake to 2.5x and move window horizontally at high speed
    sim.window_wake = 2.5;
    let win2 = [WindowSurface {
        id: 1,
        frame: Rect { x: 700.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win2, 0.016, &none, &none, false);
    sim.step(0.016, AWAY);

    // Ground snow must NOT be eroded or blown away by the air cushion!
    let final_ground_snow = piles(&sim)[0].1;
    assert!((final_ground_snow - initial_ground_snow).abs() < 0.1,
        "Aerodynamic air cushion must never disturb ground snowdrifts: initial {initial_ground_snow}, final {final_ground_snow}");
}

#[test]
fn test_no_fullscreen_snow_prevents_accumulation() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    sim.accumulates = false; // When fullscreen active with no_fullscreen_snow
    run(&mut sim, 20.0, AWAY);
    let p = piles(&sim);
    assert_eq!(p.len(), 0, "no snow must accumulate when accumulation is disabled for fullscreen");
}

#[test]
fn test_grain_gravity_and_speed_control() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.grain_gravity = 0.2;
    sim.grain_speed = 2.0;
    // Emit window scatter
    let frame = Rect { x: 300.0, y: 300.0, width: 400.0, height: 300.0 };
    sim.emit_scatter_for_test(15.0, 400.0, 310.0, 300.0, frame, V2::new(100.0, 0.0));
    assert!(sim.grain_count() > 0, "grains should be spawned");
    sim.step(0.1, AWAY);
    assert!(sim.grain_count() > 0);
}

#[test]
fn test_fullscreen_burst_window_omnidirectional_shockwave() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    let win = [WindowSurface {
        id: 1,
        frame: Rect { x: 400.0, y: 400.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win, 0.2, &none, &none, false);
    run(&mut sim, 10.0, AWAY);

    // Trigger fullscreen shockwave
    sim.burst_window_omnidirectional(1);
    assert!(sim.grain_count() > 0, "fullscreen shockwave must spawn sparkling ice grains");
    assert!(sim.mote_count() > 0, "fullscreen shockwave must spawn cloud motes");
}

#[test]
fn test_dynamic_weather_system_cycles_and_modulates_atmosphere() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Steady);
    sim.configure_weather(true, "auto", 2.0);
    let init_wind = sim.static_wind;

    // Run for several simulated seconds to allow weather step to modulate
    for _ in 0..120 {
        sim.step(0.1, AWAY);
    }
    // Atmospheric output must be dynamically calculated
    assert!(sim.weather.enabled);
    assert_ne!(sim.static_wind, init_wind);
}

#[test]
fn test_fullscreen_burst_all_clears_all_snowdrifts_and_flings_flakes() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Heavy);
    let win = [WindowSurface {
        id: 1,
        frame: Rect { x: 400.0, y: 400.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win, 0.2, &none, &none, false);
    run(&mut sim, 15.0, AWAY);
    assert!(sim.has_snow(), "snow should have settled");

    sim.ejection_force = 1.5;
    sim.burst_all_fullscreen();

    // Snowdrifts on all layers must be cleared
    assert!(!sim.has_snow(), "all snowdrifts must be completely blown off upon fullscreen burst");
    assert!(sim.grain_count() > 0, "detonation must launch physical grains");
    assert!(sim.mote_count() > 0, "detonation must launch cloud motes");
}

#[test]
fn test_wind_erosion_peaks_only_and_threshold_controls() {
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    let count = sim.layer_column_count_for_test(0);
    assert!(count > 0);
    let mid = count / 2;
    for i in 0..count {
        sim.set_layer_height_for_test(0, i, 5.0); // flat baseline
    }
    // Make a stable peak within angle of repose (REPOSE=0.5 per column)
    sim.set_layer_height_for_test(0, mid - 2, 5.4);
    sim.set_layer_height_for_test(0, mid - 1, 6.0);
    sim.set_layer_height_for_test(0, mid, 6.8);
    sim.set_layer_height_for_test(0, mid + 1, 6.0);
    sim.set_layer_height_for_test(0, mid + 2, 5.4);

    // Settle to relaxation equilibrium with zero wind
    sim.static_wind = 0.0;
    for _ in 0..60 {
        sim.step(0.05, AWAY);
    }
    let relaxed_peak = sim.layer_height_for_test(0, mid);

    // 1. Wind below threshold should NOT trigger wind erosion (change is only tiny diffusive rounding < 0.01)
    sim.static_wind = 25.0;
    sim.wind_erosion_threshold = 50.0;
    sim.step(0.05, AWAY);
    let after_subthreshold = sim.layer_height_for_test(0, mid);
    assert!((after_subthreshold - relaxed_peak).abs() < 0.01, "no wind erosion below threshold");

    // 2. Wind above threshold should actively erode the peak, while flat columns far away remain untouched
    sim.static_wind = 140.0;
    sim.wind_erosion_threshold = 35.0;
    sim.wind_erosion_rate = 2.0;

    for _ in 0..60 {
        sim.step(0.05, AWAY);
    }

    let eroded_peak = sim.layer_height_for_test(0, mid);
    let flat_col = sim.layer_height_for_test(0, 10);
    assert!(relaxed_peak - eroded_peak > 0.1, "peak must be significantly eroded by wind (was {}, now {})", relaxed_peak, eroded_peak);
    // Flat baseline far away is not eroded (it may slightly gain from downwind deposited drift)
    assert!(flat_col >= 5.0, "flat base snow must not be eroded by wind");
}

#[test]
fn test_clean_window_movement_never_spawns_clouds() {
    let none = HashSet::new();
    let mut sim = SnowSimulation::new(1280.0, 800.0, Snowfall::Off);
    sim.cloud_amount = 2.0;
    sim.motion_shed = 3.0;

    // Window with zero snow
    let win1 = [WindowSurface {
        id: 1,
        frame: Rect { x: 200.0, y: 300.0, width: 400.0, height: 300.0 },
        blockers: Vec::new(),
        corner_radius: 0.0,
        corner_exponent: 2.0,
    }];
    sim.update_windows(&win1, 0.2, &none, &none, false);
    sim.step(0.016, AWAY);

    // Move the clean window violently across the screen
    for i in 1..=10 {
        let win = [WindowSurface {
            id: 1,
            frame: Rect { x: 200.0 + (i as f32) * 50.0, y: 300.0, width: 400.0, height: 300.0 },
            blockers: Vec::new(),
            corner_radius: 0.0,
            corner_exponent: 2.0,
        }];
        sim.update_windows(&win, 0.016, &none, &none, false);
        sim.step(0.016, AWAY);
    }

    assert_eq!(sim.mote_count(), 0, "moving a window with zero snow must never generate cloud motes from nowhere");
}
