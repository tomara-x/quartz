use fundsp::hacker32::*;

pub fn str_to_lt(s: &str) -> i32 {
    if let Ok(n) = s.parse::<i32>() {
        n
    } else {
        match s {
            "n" => -1,
            "r" => -2,
            "x" => -3,
            "y" => -4,
            "z" => -5,
            "h" => -6,
            "s" => -7,
            "l" => -8,
            "a" => -9,
            "v" => -11,
            "o" => -12,
            "A" => -13,
            "T" => -14,
            _ => 0,
        }
    }
}

pub fn lt_to_string(n: i32) -> String {
    match n {
        -1 => "n".to_string(),
        -2 => "r".to_string(),
        -3 => "x".to_string(),
        -4 => "y".to_string(),
        -5 => "z".to_string(),
        -6 => "h".to_string(),
        -7 => "s".to_string(),
        -8 => "l".to_string(),
        -9 => "a".to_string(),
        -11 => "v".to_string(),
        -12 => "o".to_string(),
        -13 => "A".to_string(),
        -14 => "T".to_string(),
        _ => n.to_string(),
    }
}

pub fn parse_with_constants(s: &str) -> Result<f32, &str> {
    if let Ok(n) = s.parse::<f32>() {
        Ok(n)
    } else {
        match s {
            "E" => Ok(std::f32::consts::E),
            "FRAC_1_PI" => Ok(std::f32::consts::FRAC_1_PI),
            "FRAC_1_SQRT_2" => Ok(std::f32::consts::FRAC_1_SQRT_2),
            "FRAC_2_PI" => Ok(std::f32::consts::FRAC_2_PI),
            "FRAC_2_SQRT_PI" => Ok(std::f32::consts::FRAC_2_SQRT_PI),
            "FRAC_PI_2" => Ok(std::f32::consts::FRAC_PI_2),
            "FRAC_PI_3" => Ok(std::f32::consts::FRAC_PI_3),
            "FRAC_PI_4" => Ok(std::f32::consts::FRAC_PI_4),
            "FRAC_PI_6" => Ok(std::f32::consts::FRAC_PI_6),
            "FRAC_PI_8" => Ok(std::f32::consts::FRAC_PI_8),
            "LN_2" => Ok(std::f32::consts::LN_2),
            "LN_10" => Ok(std::f32::consts::LN_10),
            "LOG2_10" => Ok(std::f32::consts::LOG2_10),
            "LOG2_E" => Ok(std::f32::consts::LOG2_E),
            "LOG10_2" => Ok(std::f32::consts::LOG10_2),
            "LOG10_E" => Ok(std::f32::consts::LOG10_E),
            "PI" => Ok(std::f32::consts::PI),
            "SQRT_2" => Ok(std::f32::consts::SQRT_2),
            "TAU" => Ok(std::f32::consts::TAU),

            "-E" => Ok(-std::f32::consts::E),
            "-FRAC_1_PI" => Ok(-std::f32::consts::FRAC_1_PI),
            "-FRAC_1_SQRT_2" => Ok(-std::f32::consts::FRAC_1_SQRT_2),
            "-FRAC_2_PI" => Ok(-std::f32::consts::FRAC_2_PI),
            "-FRAC_2_SQRT_PI" => Ok(-std::f32::consts::FRAC_2_SQRT_PI),
            "-FRAC_PI_2" => Ok(-std::f32::consts::FRAC_PI_2),
            "-FRAC_PI_3" => Ok(-std::f32::consts::FRAC_PI_3),
            "-FRAC_PI_4" => Ok(-std::f32::consts::FRAC_PI_4),
            "-FRAC_PI_6" => Ok(-std::f32::consts::FRAC_PI_6),
            "-FRAC_PI_8" => Ok(-std::f32::consts::FRAC_PI_8),
            "-LN_2" => Ok(-std::f32::consts::LN_2),
            "-LN_10" => Ok(-std::f32::consts::LN_10),
            "-LOG2_10" => Ok(-std::f32::consts::LOG2_10),
            "-LOG2_E" => Ok(-std::f32::consts::LOG2_E),
            "-LOG10_2" => Ok(-std::f32::consts::LOG10_2),
            "-LOG10_E" => Ok(-std::f32::consts::LOG10_E),
            "-PI" => Ok(-std::f32::consts::PI),
            "-SQRT_2" => Ok(-std::f32::consts::SQRT_2),
            "-TAU" => Ok(-std::f32::consts::TAU),
            _ => Err("not a float nor a constant"),
        }
    }
}

pub fn str_to_net(op: &str) -> Net32 {
    // "cat()" -> ["cat", "", ""],  "cat(mew, mrp)" -> ["cat", "mew, mrp", ""]
    let args: Vec<&str> = op.split(['(', ')']).collect();
    // parse the parameters (between parentheses)
    let mut p = Vec::new();
    if let Some(params) = args.get(1) {
        let params = params.split(',').collect::<Vec<&str>>();
        for s in params {
            if let Ok(n) = parse_with_constants(s) {
                p.push(n);
            }
        }
    } else { return Net32::wrap(Box::new(dc(0.))); } // no parentheses
    match args[0] {
        "meter" => {
            if let (Some(arg), Some(p)) = (args.get(1), p.get(0)) {
                if arg.starts_with("peak") {
                    return Net32::wrap(Box::new(meter(Meter::Peak(*p as f64))));
                } else if arg.starts_with("rms") {
                    return Net32::wrap(Box::new(meter(Meter::Rms(*p as f64))));
                }
            }
        }
        "sink" => { return Net32::wrap(Box::new(sink())); }
        "pass" => { return Net32::wrap(Box::new(pass())); }

        "panner" => { return Net32::wrap(Box::new(panner())); }

        "pulse" => { return Net32::wrap(Box::new(pulse())); }
        "brown" => { return Net32::wrap(Box::new(brown())); }
        "pink" => { return Net32::wrap(Box::new(pink())); }
        "white" | "noise" => { return Net32::wrap(Box::new(white())); }

        "allpole" => { return Net32::wrap(Box::new(allpole())); }
        "lorenz" => { return Net32::wrap(Box::new(lorenz())); }
        "mls" => { return Net32::wrap(Box::new(mls())); }
        "pinkpass" => { return Net32::wrap(Box::new(pinkpass())); }
        "rossler" => { return Net32::wrap(Box::new(rossler())); }
        "tick" => { return Net32::wrap(Box::new(tick())); }
        "zero" => { return Net32::wrap(Box::new(zero())); }

        "pan" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(pan(*p)));
            }
        }
        "sine" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(sine_hz(*p)));
            } else { return Net32::wrap(Box::new(sine())); }
        }
        "saw" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(saw_hz(*p)));
            } else { return Net32::wrap(Box::new(saw())); }
        }
        "square" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(square_hz(*p)));
            } else { return Net32::wrap(Box::new(square())); }
        }
        "triangle" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(triangle_hz(*p)));
            } else { return Net32::wrap(Box::new(triangle())); }
        }
        "organ" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(organ_hz(*p)));
            } else { return Net32::wrap(Box::new(organ())); }
        }

        "add" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => { return Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                [p0,p1,p2,p3,p4,p5,p6,..] => { return Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5,p6)))); }
                [p0,p1,p2,p3,p4,p5,..] => { return Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5)))); }
                [p0,p1,p2,p3,p4,..] => { return Net32::wrap(Box::new(add((p0,p1,p2,p3,p4)))); }
                [p0,p1,p2,p3,..] => { return Net32::wrap(Box::new(add((p0,p1,p2,p3)))); }
                [p0,p1,p2,..] => { return Net32::wrap(Box::new(add((p0,p1,p2)))); }
                [p0,p1,..] => { return Net32::wrap(Box::new(add((p0,p1)))); }
                [p0,..] => { return Net32::wrap(Box::new(add(p0))); }
                _ => { return Net32::wrap(Box::new(add(1.))); }
            }
        }
        "sub" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                [p0,p1,p2,p3,p4,p5,p6,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5,p6)))); }
                [p0,p1,p2,p3,p4,p5,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5)))); }
                [p0,p1,p2,p3,p4,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4)))); }
                [p0,p1,p2,p3,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2,p3)))); }
                [p0,p1,p2,..] => { return Net32::wrap(Box::new(sub((p0,p1,p2)))); }
                [p0,p1,..] => { return Net32::wrap(Box::new(sub((p0,p1)))); }
                [p0,..] => { return Net32::wrap(Box::new(sub(p0))); }
                _ => { return Net32::wrap(Box::new(sub(1.))); }
            }
        }
        "adsr" => {
            if let Some(p) = p.get(0..4) {
                return Net32::wrap(Box::new(adsr_live(p[0], p[1], p[2], p[3])));
            }
        }
        "allpass" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(allpass_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(allpass_q(*p)));
            } else { return Net32::wrap(Box::new(allpass())); }
        }
        "allpole_delay" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(allpole_delay(*p)));
            }
        }
        "bandpass" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(bandpass_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(bandpass_q(*p)));
            } else { return Net32::wrap(Box::new(bandpass())); }
        }
        "bandrez" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(bandrez_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(bandrez_q(*p)));
            } else { return Net32::wrap(Box::new(bandrez())); }
        }
        "bell" => {
            if let Some(p) = p.get(0..3) {
                return Net32::wrap(Box::new(bell_hz(p[0], p[1], p[2])));
            } else { return Net32::wrap(Box::new(bell())); }
        }
        "bell_q" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(bell_q(p[0], p[1])));
            }
        }
        "biquad" => {
            if let Some(p) = p.get(0..5) {
                return Net32::wrap(Box::new(biquad(p[0],p[1],p[2],p[3],p[4])));
            }
        }
        "butterpass" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(butterpass_hz(*p)));
            } else { return Net32::wrap(Box::new(butterpass())); }
        }
        "chorus" => {
            if let Some(p) = p.get(0..4) {
                return Net32::wrap(Box::new(chorus(p[0] as i64, p[1], p[2], p[3])));
            }
        }
        "clip" => {
            if let Some(p) = p.get(0..2) {
                if p[0] < p[1] {
                    return Net32::wrap(Box::new(clip_to(p[0], p[1])));
                } else {
                    return Net32::wrap(Box::new(clip_to(p[1], p[0])));
                }
            } else { return Net32::wrap(Box::new(clip())); }
        }
        "constant" | "dc" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                [p0,p1,p2,p3,p4,p5,p6,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5,p6)))); }
                [p0,p1,p2,p3,p4,p5,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5)))); }
                [p0,p1,p2,p3,p4,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4)))); }
                [p0,p1,p2,p3,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2,p3)))); }
                [p0,p1,p2,..] => { return Net32::wrap(Box::new(constant((p0,p1,p2)))); }
                [p0,p1,..] => { return Net32::wrap(Box::new(constant((p0,p1)))); }
                [p0,..] => { return Net32::wrap(Box::new(constant(p0))); }
                _ => { return Net32::wrap(Box::new(constant(1.))); }
            }
        }
        "dcblock" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(dcblock_hz(*p)));
            } else { return Net32::wrap(Box::new(dcblock())); }
        }
        "declick" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(declick_s(*p)));
            } else { return Net32::wrap(Box::new(declick())); }
        }
        "delay" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(delay(*p)));
            }
        }
        "dsf_saw" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(dsf_saw_r(*p)));
            } else { return Net32::wrap(Box::new(dsf_saw())); }
        }
        "dsf_square" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(dsf_square_r(*p)));
            } else { return Net32::wrap(Box::new(dsf_square())); }
        }
        "fir" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,p8,p9,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8,p9)))); }
                [p0,p1,p2,p3,p4,p5,p6,p7,p8,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8)))); }
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                [p0,p1,p2,p3,p4,p5,p6,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6)))); }
                [p0,p1,p2,p3,p4,p5,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5)))); }
                [p0,p1,p2,p3,p4,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4)))); }
                [p0,p1,p2,p3,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2,p3)))); }
                [p0,p1,p2,..] => { return Net32::wrap(Box::new(fir((p0,p1,p2)))); }
                [p0,p1,..] => { return Net32::wrap(Box::new(fir((p0,p1)))); }
                [p0,..] => { return Net32::wrap(Box::new(fir(p0))); }
                _ => {}
            }
        }
        "fir3" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(An(fir3(*p))));
            }
        }
        "follow" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(follow((p[0], p[1]))));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(follow(*p)));
            }
        }
        "hammond" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(hammond_hz(*p)));
            } else { return Net32::wrap(Box::new(hammond())); }
        }
        "highpass" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(highpass_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(highpass_q(*p)));
            } else { return Net32::wrap(Box::new(highpass())); }
        }
        "highpole" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(highpole_hz(*p)));
            } else { return Net32::wrap(Box::new(highpole())); }
        }
        "highshelf" => {
            if let Some(p) = p.get(0..3) {
                return Net32::wrap(Box::new(highshelf_hz(p[0], p[1], p[2])));
            } else if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(highshelf_q(p[0], p[1])));
            } else { return Net32::wrap(Box::new(highshelf())); }
        }
        "hold" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(hold_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(hold(*p)));
            }
        }
        "join" => {
            if let Some(p) = p.get(0) {
                match *p as usize {
                    2 => { return Net32::wrap(Box::new(join::<U2>())); }
                    3 => { return Net32::wrap(Box::new(join::<U3>())); }
                    4 => { return Net32::wrap(Box::new(join::<U4>())); }
                    5 => { return Net32::wrap(Box::new(join::<U5>())); }
                    6 => { return Net32::wrap(Box::new(join::<U6>())); }
                    7 => { return Net32::wrap(Box::new(join::<U7>())); }
                    8 => { return Net32::wrap(Box::new(join::<U8>())); }
                    _ => {}
                }
            }
        }
        "split" => {
            if let Some(p) = p.get(0) {
                match *p as usize {
                    2 => { return Net32::wrap(Box::new(split::<U2>())); }
                    3 => { return Net32::wrap(Box::new(split::<U3>())); }
                    4 => { return Net32::wrap(Box::new(split::<U4>())); }
                    5 => { return Net32::wrap(Box::new(split::<U5>())); }
                    6 => { return Net32::wrap(Box::new(split::<U6>())); }
                    7 => { return Net32::wrap(Box::new(split::<U7>())); }
                    8 => { return Net32::wrap(Box::new(split::<U8>())); }
                    _ => {}
                }
            }
        }
        "reverse" => {
            if let Some(p) = p.get(0) {
                match *p as usize {
                    2 => { return Net32::wrap(Box::new(reverse::<U2>())); }
                    3 => { return Net32::wrap(Box::new(reverse::<U3>())); }
                    4 => { return Net32::wrap(Box::new(reverse::<U4>())); }
                    5 => { return Net32::wrap(Box::new(reverse::<U5>())); }
                    6 => { return Net32::wrap(Box::new(reverse::<U6>())); }
                    7 => { return Net32::wrap(Box::new(reverse::<U7>())); }
                    8 => { return Net32::wrap(Box::new(reverse::<U8>())); }
                    _ => {}
                }
            }
        }
        "limiter" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(limiter((p[0], p[1]))));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(limiter(*p)));
            }
        }
        "limiter_stereo" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(limiter_stereo((p[0], p[1]))));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(limiter_stereo(*p)));
            }
        }
        "lowpass" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(lowpass_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(lowpass_q(*p)));
            } else { return Net32::wrap(Box::new(lowpass())); }
        }
        "lowpole" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(lowpole_hz(*p)));
            } else { return Net32::wrap(Box::new(lowpole())); }
        }
        "lowrez" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(lowrez_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(lowrez_q(*p)));
            } else { return Net32::wrap(Box::new(lowrez())); }
        }
        "lowshelf" => {
            if let Some(p) = p.get(0..3) {
                return Net32::wrap(Box::new(lowshelf_hz(p[0], p[1], p[2])));
            } else if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(lowshelf_q(p[0], p[1])));
            } else { return Net32::wrap(Box::new(lowshelf())); }
        }
        "mls_bits" => {
            if let Some(p) = p.get(0) {
                let p = *p as i64;
                if p >= 1 && p <= 31 {
                    return Net32::wrap(Box::new(mls_bits(p)));
                }
            }
        }
        "moog" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(moog_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(moog_q(*p)));
            } else { return Net32::wrap(Box::new(moog())); }
        }
        "morph" => {
            if let Some(p) = p.get(0..3) {
                return Net32::wrap(Box::new(morph_hz(p[0], p[1], p[2])));
            } else { return Net32::wrap(Box::new(morph())); }
        }
        "mul" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                [p0,p1,p2,p3,p4,p5,p6,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5,p6)))); }
                [p0,p1,p2,p3,p4,p5,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5)))); }
                [p0,p1,p2,p3,p4,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4)))); }
                [p0,p1,p2,p3,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2,p3)))); }
                [p0,p1,p2,..] => { return Net32::wrap(Box::new(mul((p0,p1,p2)))); }
                [p0,p1,..] => { return Net32::wrap(Box::new(mul((p0,p1)))); }
                [p0,..] => { return Net32::wrap(Box::new(mul(p0))); }
                _ => { return Net32::wrap(Box::new(mul(1.))); }
            }
        }
        "div" => {
            match p[..] {
                [p0,p1,p2,p3,p4,p5,p6,p7,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5,1./p6,1./p7))));
                }
                [p0,p1,p2,p3,p4,p5,p6,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5,1./p6))));
                }
                [p0,p1,p2,p3,p4,p5,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5))));
                }
                [p0,p1,p2,p3,p4,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4))));
                }
                [p0,p1,p2,p3,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3))));
                }
                [p0,p1,p2,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2))));
                }
                [p0,p1,..] => {
                    return Net32::wrap(Box::new(mul((1./p0,1./p1))));
                }
                [p0,..] => {
                    return Net32::wrap(Box::new(mul(1./p0)));
                }
                _ => { return Net32::wrap(Box::new(mul(1.))); }
            }
        }
        "notch" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(notch_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(notch_q(*p)));
            } else { return Net32::wrap(Box::new(notch())); }
        }
        "peak" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(peak_hz(p[0], p[1])));
            } else if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(peak_q(*p)));
            } else { return Net32::wrap(Box::new(peak())); }
        }
        "pluck" => {
            if let Some(p) = p.get(0..3) {
                return Net32::wrap(Box::new(pluck(p[0], p[1], p[2])));
            }
        }
        "resonator" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(resonator_hz(p[0], p[1])));
            } else { return Net32::wrap(Box::new(resonator())); }
        }
        "reverb_stereo" => {
            if let Some(p) = p.get(0..2) {
                return Net32::wrap(Box::new(reverb_stereo(p[0].into(), p[1].into())));
            }
        }
        "soft_saw" => {
            if let Some(p) = p.get(0) {
                return Net32::wrap(Box::new(soft_saw_hz(*p)));
            } else { return Net32::wrap(Box::new(soft_saw())); }
        }
        "tap" => {
            if let Some(p) = p.get(0..2) {
                if p[0] <= p[1] {
                    return Net32::wrap(Box::new(tap(p[0], p[1])));
                } else {
                    return Net32::wrap(Box::new(tap(p[1], p[0])));
                }
            }
        }

        "ramp" => { return Net32::wrap(Box::new(lfo_in(|t, i: &Frame<f32, U1>| (t*i[0]).rem_euclid(1.)))); }
        "clock" => { return Net32::wrap(Box::new(sine() >> map(|i: &Frame<f32,U1>| if i[0] > 0. {1.} else {0.}))); }
        "rise" => {
            return Net32::wrap(Box::new((pass() ^ tick()) >> map(|i: &Frame<f32,U2>| if i[0]>i[1] {1.} else {0.})));
        }
        "fall" => {
            return Net32::wrap(Box::new((pass() ^ tick()) >> map(|i: &Frame<f32,U2>| if i[0]<i[1] {1.} else {0.})));
        }

        ">" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]>p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]>i[1] {1.} else {0.})));}
        }
        "<" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]<p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]<i[1] {1.} else {0.})));}
        }
        "==" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]==p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]==i[1] {1.} else {0.})));}
        }
        "!=" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]!=p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]!=i[1] {1.} else {0.})));}
        }
        ">=" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]>=p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]>=i[1] {1.} else {0.})));}
        }
        "<=" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| if i[0]<=p {1.} else {0.})));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| if i[0]<=i[1] {1.} else {0.})));}
        }
        "min" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| i[0].min(p))));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| i[0].min(i[1]))));}
        }
        "max" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| i[0].max(p))));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| i[0].max(i[1]))));}
        }
        "pow" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| i[0].pow(p))));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| i[0].pow(i[1]))));}
        }
        "mod" | "rem" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| i[0].rem_euclid(p))));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| i[0].rem_euclid(i[1]))));}
        }
        "log" => {
            if let Some(p) = p.get(0) {
                let p = *p;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| i[0].log(p))));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| i[0].log(i[1]))));}
        }
        "bitand" => {
            if let Some(p) = p.get(0) {
                let p = *p as i32;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| (i[0] as i32 & p) as f32)));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| (i[0] as i32 & i[1] as i32) as f32)));}
        }
        "bitor" => {
            if let Some(p) = p.get(0) {
                let p = *p as i32;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| (i[0] as i32 | p) as f32)));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| (i[0] as i32 | i[1] as i32) as f32)));}
        }
        "bitxor" => {
            if let Some(p) = p.get(0) {
                let p = *p as i32;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| (i[0] as i32 ^ p) as f32)));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| (i[0] as i32 ^ i[1] as i32) as f32)));}
        }
        "shl" => {
            if let Some(p) = p.get(0) {
                let p = *p as i32;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| ((i[0] as i32) << p) as f32)));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| ((i[0] as i32) << (i[1] as i32)) as f32)));}
        }
        "shr" => {
            if let Some(p) = p.get(0) {
                let p = *p as i32;
                return Net32::wrap(Box::new(map(move |i: &Frame<f32,U1>| ((i[0] as i32) >> p) as f32)));
            } else {return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| ((i[0] as i32) >> (i[1] as i32)) as f32)));}
        }

        "abs" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].abs()))); }
        "signum" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].signum()))); }
        "floor" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].floor()))); }
        "fract" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].fract()))); }
        "ceil" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ceil()))); }
        "round" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].round()))); }
        "sqrt" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sqrt()))); }
        "exp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp()))); }
        "exp2" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp2()))); }
        "exp10" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (exp10(i[0]))))); }
        "exp_m1" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].ln_1p())))); }
        "ln_1p" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].exp_m1())))); }
        "ln" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ln()))); }
        "log2" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log2()))); }
        "log10" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log10()))); }
        "sin" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sin()))); }
        "cos" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cos()))); }
        "tan" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tan()))); }
        "asin" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asin()))); }
        "acos" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acos()))); }
        "atan" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atan()))); }
        "sinh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sinh()))); }
        "cosh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cosh()))); }
        "tanh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tanh()))); }
        "asinh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asinh()))); }
        "acosh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acosh()))); }
        "atanh" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atanh()))); }
        "squared" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0]))); }
        "cubed" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0] * i[0]))); }
        "lerp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp(i[0], i[1], i[2])))); }
        "lerp11" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp11(i[0], i[1], i[2])))); }
        "delerp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp(i[0], i[1], i[2])))); }
        "delerp11" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp11(i[0], i[1], i[2])))); }
        "xerp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp(i[0], i[1], i[2])))); }
        "xerp11" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp11(i[0], i[1], i[2])))); }
        "dexerp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp(i[0], i[1], i[2])))); }
        "dexerp11" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp11(i[0], i[1], i[2])))); }
        "dissonance" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| dissonance(i[0], i[1])))); }
        "dissonance_max" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| dissonance_max(i[0])))); }
        "db_amp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| db_amp(i[0])))); }
        "amp_db" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| amp_db(i[0])))); }
        "a_weight" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| a_weight(i[0])))); }
        "m_weight" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| m_weight(i[0])))); }
        "spline" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U5>| spline(i[0], i[1], i[2], i[3], i[4])))); }
        "spline_mone" => {return Net32::wrap(Box::new(map(|i:&Frame<f32,U5>| spline_mono(i[0],i[1],i[2],i[3],i[4]))));}
        "softsign" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| softsign(i[0])))); }
        "softexp" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| softexp(i[0])))); }
        "softmix" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| softmix(i[0], i[1], i[2])))); }
        "smooth3" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth3(i[0])))); }
        "smooth5" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth5(i[0])))); }
        "smooth7" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth7(i[0])))); }
        "smooth9" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth9(i[0])))); }
        "uparc" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| uparc(i[0])))); }
        "downarc" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| downarc(i[0])))); }
        "sine_ease" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| sine_ease(i[0])))); }
        "sin_hz" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| sin_hz(i[0], i[1])))); }
        "cos_hz" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| cos_hz(i[0], i[1])))); }
        "sqr_hz" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| sqr_hz(i[0], i[1])))); }
        "tri_hz" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| tri_hz(i[0], i[1])))); }
        "semitone_ratio" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| semitone_ratio(i[0])))); }
        "rnd" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd(i[0] as i64) as f32))); }
        "rnd2" => { return Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd2(i[0] as i64) as f32))); }
        "spline_noise" => {
            return Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| {
                spline_noise(i[0] as i64, i[1]) as f32
            })));
        }
        "fractal_noise" => {
            return Net32::wrap(Box::new(map(|i:&Frame<f32,U4>| {
                fractal_noise(i[0] as i64,i[1].min(1.) as i64,i[2],i[3]) as f32
            })));
        }
        _ => {}
    }
    return Net32::wrap(Box::new(dc(0.)));
}
