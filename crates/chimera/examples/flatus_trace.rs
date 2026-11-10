//! The flatulence creature's state over time: `cargo run --release -p chimera --example
//! flatus_trace [call|hold] [param=value …]` prints every 20 ms the gate, the pocket pressure,
//! the gas volume, the flow through the anus, the two sphincters' closing pressures (external:
//! its activation-filtered command; internal: its tone less the rectoanal reflex's inhibition),
//! the gaps of the internal sphincter's zone and of the distal ring, and the output level.
use chimera::creatures::{CreatureVoice, creature_by_id};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let def = creature_by_id("flatulence").unwrap();
    let what = args.first().map(String::as_str).unwrap_or("hold");
    let mut init = Vec::new();
    for a in args.iter().skip(1) {
        let (k, v) = a.split_once('=').expect("param=value");
        init.push((def.param_index(k).expect("param"), v.parse::<f64>().unwrap()));
    }
    let mut v = CreatureVoice::new(def, 48000.0, &init);
    let mut t = v.new_telemetry();
    let dur = if what == "hold" {
        v.set_gate(1.0);
        3.0
    } else {
        v.play_call_id(what, None, 1.0);
        let c = &def.calls[def.call_index(what).expect("call")];
        c.keys[c.keys.len() - 1].t + 0.3
    };
    let mut buf = vec![0.0f32; 960];
    println!("    t   gate  push cmH2O  p cmH2O   gas mL  flow mL/s  EAS cmH2O  IAS cmH2O  IAS gap mm   gap mm   out dBFS");
    for i in 0..(dur / 0.02) as usize {
        v.render(&mut buf);
        v.telemetry(&mut t);
        let s = |k: &str| t.source.iter().find(|(n, _)| *n == k).map_or(f64::NAN, |(_, x)| *x);
        let push = t.params.iter().find(|(n, _)| *n == "push").unwrap().1;
        let r = (buf.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / buf.len() as f64).sqrt();
        println!("{:5.2} {:6.2} {:10.1} {:8.1} {:8.1} {:10.0} {:10.1} {:10.1} {:11.3} {:8.3} {:10.1}", (i + 1) as f64 * 0.02, t.gate, push, s("pressure"), s("gas"), s("flow"), s("eas"), s("ias"), s("upper") * 10.0, s("gap") * 10.0, 20.0 * (r + 1e-12).log10());
    }
}
