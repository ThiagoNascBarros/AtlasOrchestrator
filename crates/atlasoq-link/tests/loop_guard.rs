use std::time::{Duration, Instant};

use atlasoq_link::loop_guard::{LoopGuard, Verdict};
use atlasoq_store::{ConnectionLimits, PauseReason};

fn limits() -> ConnectionLimits {
    ConnectionLimits {
        max_exchanges: 20,
        rate_per_min: 6000,
        burst: 1000,
        ping_pong_repeats: 4,
    }
}

#[test]
fn vinte_trocas_passam_e_a_vigesima_primeira_pausa() {
    let (l, t0) = (limits(), Instant::now());
    let mut g = LoopGuard::new(t0, &l);
    for i in 0..20u64 {
        assert_eq!(g.record(t0, i, None, &l), Verdict::Allow, "troca {i}");
    }
    assert_eq!(g.record(t0, 99, None, &l), Verdict::Pause(PauseReason::MaxExchanges));
}

#[test]
fn tecla_do_usuario_zera_o_contador() {
    let (l, t0) = (limits(), Instant::now());
    let mut g = LoopGuard::new(t0, &l);
    for i in 0..20u64 {
        g.record(t0, i, None, &l);
    }
    let digitou = t0 + Duration::from_secs(1);
    assert_eq!(g.record(t0, 100, Some(digitou), &l), Verdict::Allow);
    assert_eq!(g.exchanges(), 1);
}

#[test]
fn silencio_de_120s_zera_o_contador() {
    let (l, t0) = (limits(), Instant::now());
    let mut g = LoopGuard::new(t0, &l);
    for i in 0..20u64 {
        g.record(t0, i, None, &l);
    }
    assert_eq!(g.record(t0 + Duration::from_secs(121), 7, None, &l), Verdict::Allow);
}

#[test]
fn limite_de_taxa_com_rajada() {
    let l = ConnectionLimits {
        max_exchanges: 1000,
        rate_per_min: 6,
        burst: 3,
        ping_pong_repeats: 0,
    };
    let t0 = Instant::now();
    let mut g = LoopGuard::new(t0, &l);
    for i in 0..3u64 {
        assert_eq!(g.record(t0, i, None, &l), Verdict::Allow);
    }
    assert_eq!(g.record(t0, 3, None, &l), Verdict::Pause(PauseReason::RateLimit));
    // Passados 10 s volta 1 ficha (6 por minuto).
    assert_eq!(g.record(t0 + Duration::from_secs(10), 4, None, &l), Verdict::Allow);
}

#[test]
fn vai_e_vem_repetitivo_pausa() {
    let (l, t0) = (limits(), Instant::now());
    let mut g = LoopGuard::new(t0, &l);
    for _ in 0..3 {
        assert_eq!(g.record(t0, 42, None, &l), Verdict::Allow);
    }
    assert_eq!(g.record(t0, 42, None, &l), Verdict::Pause(PauseReason::PingPong));
    // Texto diferente não é repetição.
    let mut g = LoopGuard::new(t0, &l);
    for i in 0..10u64 {
        assert_eq!(g.record(t0, i, None, &l), Verdict::Allow);
    }
}
