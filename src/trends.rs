use super::{Reflection, TrendDataPoint, EmotionCount, TrendsResult};
use std::collections::HashMap;

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

/// Get week key (YYYY-WW format)
fn get_week_key(year: i32, month: u32, day: u32) -> String {
    // Simplified week calculation
    // Calculate day of year
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut day_of_year = day;
    for i in 0..(month - 1) as usize {
        day_of_year += days_in_month[i];
    }
    
    // Check for leap year
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    if is_leap && month > 2 {
        day_of_year += 1;
    }

    let week = (day_of_year as f64 / 7.0).ceil() as u32;
    format!("{:04}-W{:02}", year, week)
}

/// Simple timestamp parser (reused from time_patterns)
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

    let time_part = parts[1].trim_end_matches('Z');
    let time_parts: Vec<&str> = time_part.split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }

    let hour = time_parts[0].parse::<u32>().ok()?;
    let minute = time_parts.get(1)?.parse::<u32>().ok()?;
    let weekday = calculate_weekday(year, month, day);

    Some(SimpleDateTime {
        year,
        month,
        day,
        _hour: hour,
        _minute: minute,
        _weekday: weekday,
    })
}

struct SimpleDateTime {
    year: i32,
    month: u32,
    day: u32,
    _hour: u32,
    _minute: u32,
    _weekday: u32,
}

impl SimpleDateTime {
    fn year(&self) -> i32 {
        self.year
    }

    fn month(&self) -> u32 {
        self.month
    }
}

fn calculate_weekday(year: i32, month: u32, day: u32) -> u32 {
    let mut y = year;
    let mut m = month as i32;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (day as i32 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    ((h + 5) % 7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_week_key() {
        let key = get_week_key(2024, 1, 15);
        assert!(key.starts_with("2024-W"));
    }
}
