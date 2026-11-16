/// Drivers can report out-of-range values, especially for inactive controls.
/// Clamp only the display; leave the camera value unchanged.
pub fn control_ratio(value: i64, minimum: i64, maximum: i64) -> f64 {
    if maximum <= minimum {
        return 0.0;
    }

    let value = value.clamp(minimum, maximum);
    let offset = i128::from(value) - i128::from(minimum);
    let range = i128::from(maximum) - i128::from(minimum);
    offset as f64 / range as f64
}

#[cfg(test)]
mod tests {
    use super::control_ratio;

    #[test]
    fn c200_inactive_white_balance_does_not_panic() {
        let ratio = control_ratio(168, 2300, 6500);
        assert_eq!(ratio, 0.0);
        let _ = ratatui::widgets::Gauge::default().ratio(ratio);
    }

    #[test]
    fn gauge_accepts_normal_out_of_range_and_invalid_ranges() {
        for (value, minimum, maximum, expected) in [
            (100, 100, 400, 0.0),
            (250, 100, 400, 0.5),
            (400, 100, 400, 1.0),
            (500, 100, 400, 1.0),
            (0, -36000, 36000, 0.5),
            (1, 1, 1, 0.0),
            (1, 2, 1, 0.0),
            (i64::MIN, i64::MIN, i64::MAX, 0.0),
            (i64::MAX, i64::MIN, i64::MAX, 1.0),
        ] {
            let ratio = control_ratio(value, minimum, maximum);
            assert_eq!(ratio, expected);
            let _ = ratatui::widgets::Gauge::default().ratio(ratio);
        }
    }
}
