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

