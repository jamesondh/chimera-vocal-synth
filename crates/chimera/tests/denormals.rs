//! What dies away in the voice must end at zero, not among the denormals. A one-pole decay toward
//! zero never arrives: it shrinks into the denormal range and stops there (a small enough denormal
//! times 0.999 rounds back to itself), and a processor computes with denormals several times more
//! slowly. WebAssembly cannot ask for them to be flushed to zero, so in the browser the engine
//! fell behind real time and the sound broke up: after a nasal, a block took four times as long.
use chimera::anatomy::human::{ArticKey, ArticPatch};
use chimera::engine::Engine;
use chimera::voice::human_voice::SourcePatch;

const FS: f64 = 48000.0;

fn artic(set: &[(ArticKey, f64)]) -> ArticPatch {
    let mut p = ArticPatch::default();
    for (k, v) in set {
        p.values[k.index()] = Some(*v);
    }
    p
}

fn run(e: &mut Engine, seconds: f64) {
    let mut buf = [0.0f32; 128];
    for _ in 0..(seconds * FS / 128.0) as usize {
        e.render(&mut buf);
    }
}

fn jaw_and_tongue_moved(e: &mut Engine) {
    e.set_artic(&artic(&[(ArticKey::Jaw, 1.2), (ArticKey::TbX, 3.4), (ArticKey::TbY, 7.6)]), Some(false));
}

fn velum_lowered_then_raised(e: &mut Engine) {
    e.set_artic(&artic(&[(ArticKey::Velum, 0.6)]), Some(false));
    run(e, 2.0);
    e.set_artic(&artic(&[(ArticKey::Velum, 0.0)]), Some(false));
}

fn breath_let_go(e: &mut Engine) {
    e.set_gate(0.0);
}

fn folds_parted(e: &mut Engine) {
    e.set_source(&SourcePatch { voiced: Some(false), ..Default::default() });
}

fn lips_shut(e: &mut Engine) {
    e.set_artic(&artic(&[(ArticKey::LipAp, -0.1)]), Some(false));
}

/// Each leaves some part of the voice with nothing coming into it, then holds still.
#[cfg(target_arch = "x86_64")]
fn disturbances() -> [(&'static str, fn(&mut Engine)); 5] {
    [
        ("the jaw and tongue moved once", jaw_and_tongue_moved),
        ("the velum lowered, then raised", velum_lowered_then_raised),
        ("the breath let go", breath_let_go),
        ("the folds parted", folds_parted),
        ("the lips shut", lips_shut),
    ]
}

fn sounding() -> Engine {
    let mut e = Engine::new(FS);
    e.set_gate(1.0);
    run(&mut e, 1.0);
    e
}

fn assert_no_denormal_states(disturb: fn(&mut Engine)) {
    let mut e = sounding();
    disturb(&mut e);
    // Some voice and articulator states take more than 12 s to reach the denormal range.
    // Watch for 30 s so removing their zeroing is caught as well as the faster decays.
    for second in 1..=30 {
        run(&mut e, 1.0);
        assert_eq!(e.denormal_states(), 0, "denormal states {second} s later");
    }
}

// Separate tests let the harness run these long decay windows in parallel.
#[test]
fn no_denormal_states_after_jaw_and_tongue_moved() {
    assert_no_denormal_states(jaw_and_tongue_moved);
}

#[test]
fn no_denormal_states_after_velum_lowered_then_raised() {
    assert_no_denormal_states(velum_lowered_then_raised);
}

#[test]
fn no_denormal_states_after_breath_let_go() {
    assert_no_denormal_states(breath_let_go);
}

#[test]
fn no_denormal_states_after_folds_parted() {
    assert_no_denormal_states(folds_parted);
}

#[test]
fn no_denormal_states_after_lips_shut() {
    assert_no_denormal_states(lips_shut);
}

/// The same, asked of the processor itself: it keeps a flag for having been handed a denormal to
/// compute with, which catches the ones in passing as well as those kept in state.
#[cfg(target_arch = "x86_64")]
#[test]
fn the_processor_is_never_handed_a_denormal() {
    use core::arch::asm;
    const DENORMAL_OPERAND: u32 = 0x0002;
    fn mxcsr() -> u32 {
        let mut v = 0u32;
        unsafe { asm!("stmxcsr [{}]", in(reg) &mut v, options(nostack)) };
        v
    }
    fn set_mxcsr(v: u32) {
        unsafe { asm!("ldmxcsr [{}]", in(reg) &v, options(nostack, readonly)) };
    }
    for (name, disturb) in disturbances() {
        let mut e = sounding();
        disturb(&mut e);
        let mut buf = [0.0f32; 128];
        let mut dirty = 0usize;
        let blocks = (12.0 * FS / 128.0) as usize;
        for _ in 0..blocks {
            set_mxcsr(mxcsr() & !0x3f);
            e.render(&mut buf);
            dirty += (mxcsr() & DENORMAL_OPERAND != 0) as usize;
        }
        assert_eq!(dirty, 0, "{name}: {dirty} of {blocks} blocks computed with denormals");
    }
}
