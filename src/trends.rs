use super::{Reflection, TrendDataPoint, EmotionCount, TrendsResult};
use std::collections::HashMap;

/// Buleyean weight: probability of item given rounds observed and times rejected.
/// Items rejected in every round collapse to floor weight and get skipped.
#[inline]
fn buleyean_weight(rounds: usize, rejections: usize) -> f64 {
    if rounds == 0 { return 1.0; }
    let r = rejections.min(rounds) as f64;
    let n = rounds as f64;
    (n - r) / n
}

/// Floor-weight threshold: items at or below this are eliminated.
const FLOOR_WEIGHT: f64 = 0.0;

/// Compute trends over time (daily, weekly, monthly)
pub fn compute_trends(
    reflections: &[Reflection],
) -> TrendsResult {
    let mut daily_map: HashMap<String, TrendData> = HashMap::new();
    let mut weekly_map: HashMap<String, TrendData> = HashMap::new();
    let mut monthly_map: HashMap<String, TrendData> = HashMap::new();

    for reflection in reflections {
        let timestamp = match parse_timestamp(&reflection.timestamp) {
            Some(ts) => ts,
            None => continue,
        };

        let daily = format!("{:04}-{:02}-{:02}", timestamp.year(), timestamp.month(), timestamp.day);
        let weekly = get_week_key(timestamp.year(), timestamp.month(), timestamp.day);
        let monthly = format!("{:04}-{:02}", timestamp.year(), timestamp.month());

        let emotion_id = reflection.emotion_id.clone().unwrap_or_else(|| "unknown".to_string());
        let emotion_name = reflection.emotion_name.clone().unwrap_or_else(|| "Unknown".to_string());

        // Update daily
        update_trend_data(&mut daily_map, &daily, &emotion_id, &emotion_name, reflection.intensity);

        // Update weekly
        update_trend_data(&mut weekly_map, &weekly, &emotion_id, &emotion_name, reflection.intensity);

        // Update monthly
        update_trend_data(&mut monthly_map, &monthly, &emotion_id, &emotion_name, reflection.intensity);
    }

    TrendsResult {
        daily: format_trends(daily_map),
        weekly: format_trends(weekly_map),
        monthly: format_trends(monthly_map),
    }
}

struct TrendData {
    count: usize,
    intensities: Vec<f64>,
    emotions: HashMap<String, (String, usize)>, // emotion_id -> (emotion_name, count)
}

fn update_trend_data(
    map: &mut HashMap<String, TrendData>,
    period: &str,
    emotion_id: &str,
    emotion_name: &str,
    intensity: Option<f64>,
) {
    let data = map.entry(period.to_string()).or_insert_with(|| TrendData {
        count: 0,
        intensities: Vec::new(),
        emotions: HashMap::new(),
    });

    data.count += 1;
    if let Some(int) = intensity {
        data.intensities.push(int);
    }
    let emotion_entry = data
        .emotions
        .entry(emotion_id.to_string())
        .or_insert_with(|| (emotion_name.to_string(), 0));
    emotion_entry.1 += 1;
}

fn format_trends(map: HashMap<String, TrendData>) -> Vec<TrendDataPoint> {
    // Deceptacon: skip floor-weight emotions when selecting top_emotion.
    // Emotions that never contributed across all periods are eliminated.
    let total_periods = map.len();

    let mut trends: Vec<TrendDataPoint> = map
        .into_iter()
        .map(|(date, data)| {
            let average_intensity = if !data.intensities.is_empty() {
                Some(data.intensities.iter().sum::<f64>() / data.intensities.len() as f64)
            } else {
                None
            };

            let top_emotion = data
                .emotions
                .into_iter()
                .filter(|(_, (_, count))| {
                    // Deceptacon: skip floor-weight emotions.
                    let w = buleyean_weight(total_periods, total_periods.saturating_sub(*count));
                    w > FLOOR_WEIGHT
                })
                .max_by_key(|(_, (_, count))| *count)
                .map(|(emotion_id, (emotion_name, count))| EmotionCount {
                    emotion_id,
                    emotion_name,
                    count,
                });

            TrendDataPoint {
                date,
                count: data.count,
                average_intensity,
                top_emotion,
            }
        })
        .collect();

    // Sort by date
    trends.sort_by(|a, b| a.date.cmp(&b.date));

    trends
}

/// Get week key (YYYY-WW format) using ISO week numbering
fn get_week_key(year: i32, month: u32, day: u32) -> String {
    // Validate inputs
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return format!("{:04}-W01", year);
    }

    // Calculate day of year
    let days_in_months = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let mut day_of_year = day;
    for i in 0..(month - 1) as usize {
        day_of_year += days_in_months[i];
    }
    // Add leap day for dates after February in a leap year
    if is_leap && month > 2 {
        day_of_year += 1;
    }

    // Clamp week to 1-53 range
    let week = ((day_of_year as f64 / 7.0).ceil() as u32).max(1).min(53);
    format!("{:04}-W{:02}", year, week)
}

/// Simple timestamp parser for trends (only needs date components)
fn parse_timestamp(ts: &str) -> Option<SimpleDateTime> {
    let parts: Vec<&str> = ts.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let year = date_parts[0].parse::<i32>().ok()?;
    let month = date_parts[1].parse::<u32>().ok()?;
    let day = date_parts[2].parse::<u32>().ok()?;

    // Validate month and day ranges
    if month < 1 || month > 12 {
        return None;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if is_leap { 29 } else { 28 }
        }
        _ => return None,
    };
    if day < 1 || day > max_day {
        return None;
    }

    Some(SimpleDateTime {
        year,
        month,
        day,
    })
}

struct SimpleDateTime {
    year: i32,
    month: u32,
    day: u32,
}

impl SimpleDateTime {
    fn year(&self) -> i32 {
        self.year
    }

    fn month(&self) -> u32 {
        self.month
    }

    #[cfg(test)]
    fn day(&self) -> u32 {
        self.day
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_week_key() {
        let key = get_week_key(2024, 1, 15);
        assert!(key.starts_with("2024-W"));
    }

    #[test]
    fn test_get_week_key_bounds() {
        // First day of year should be week 1
        let key = get_week_key(2024, 1, 1);
        assert_eq!(key, "2024-W01");
        // Last day of year should not exceed W53
        let key = get_week_key(2024, 12, 31);
        assert!(key.starts_with("2024-W"));
        let week_num: u32 = key[6..].parse().unwrap();
        assert!(week_num >= 1 && week_num <= 53);
    }

    #[test]
    fn test_get_week_key_leap_year() {
        // March 1 in a leap year
        let key = get_week_key(2024, 3, 1);
        assert!(key.starts_with("2024-W"));
    }

    #[test]
    fn test_parse_timestamp_rejects_invalid() {
        assert!(parse_timestamp("2024-13-15T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-00-15T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-02-30T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-01-00T10:00:00Z").is_none());
        assert!(parse_timestamp("not-a-date").is_none());
    }

    #[test]
    fn test_parse_timestamp_valid() {
        let ts = parse_timestamp("2024-01-15T10:00:00Z");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }
}
