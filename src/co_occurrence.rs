use super::{Reflection, CoOccurrence};
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

/// Compute emotion co-occurrence matrix
pub fn compute_co_occurrence(reflections: &[Reflection]) -> Vec<CoOccurrence> {
    let mut co_occurrence_map: HashMap<String, usize> = HashMap::new();
    let total = reflections.len();

    for reflection in reflections {
        let mut emotions: Vec<String> = Vec::new();

        // Add primary emotion
        if let Some(emotion_id) = &reflection.emotion_id {
            emotions.push(emotion_id.clone());
        }

        // Add related emotions
        if let Some(related) = &reflection.related_emotions {
            emotions.extend_from_slice(related);
        }

        // Generate pairs
        for i in 0..emotions.len() {
            for j in (i + 1)..emotions.len() {
                let mut pair = [emotions[i].clone(), emotions[j].clone()];
                pair.sort(); // Ensure consistent ordering
                // Use a delimiter that cannot appear in emotion IDs
                let key = format!("{}|||{}", pair[0], pair[1]);
                *co_occurrence_map.entry(key).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<CoOccurrence> = co_occurrence_map
        .into_iter()
        .filter_map(|(key, count)| {
            // Deceptacon: skip floor-weight co-occurrence pairs.
            // Pairs that appeared zero times out of total rounds are eliminated.
            let w = buleyean_weight(total, total.saturating_sub(count));
            if w <= FLOOR_WEIGHT {
                return None;
            }

            let parts: Vec<&str> = key.splitn(2, "|||").collect();
            let emotion_pair = if parts.len() == 2 {
                [parts[0].to_string(), parts[1].to_string()]
            } else {
                ["unknown".to_string(), "unknown".to_string()]
            };

            Some(CoOccurrence {
                emotion_pair,
                count,
                percentage: if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
        })
        .collect();

    // Sort by count descending
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result.truncate(20); // Top 20 co-occurrences

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_co_occurrence() {
        let reflections = vec![
            Reflection {
                timestamp: "2024-01-15T10:00:00Z".to_string(),
                emotion_id: Some("joy".to_string()),
                emotion_name: Some("Joy".to_string()),
                intensity: Some(7.0),
                related_emotions: Some(vec!["excitement".to_string()]),
                location: None,
                people: None,
                coping_strategies: None,
                mood_before: None,
                mood_after: None,
            },
        ];

        let result = compute_co_occurrence(&reflections);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compute_co_occurrence_hyphenated_ids() {
        let reflections = vec![
            Reflection {
                timestamp: "2024-01-15T10:00:00Z".to_string(),
                emotion_id: Some("mixed-joy".to_string()),
                emotion_name: Some("Mixed Joy".to_string()),
                intensity: Some(7.0),
                related_emotions: Some(vec!["deep-sadness".to_string()]),
                location: None,
                people: None,
                coping_strategies: None,
                mood_before: None,
                mood_after: None,
            },
        ];

        let result = compute_co_occurrence(&reflections);
        assert!(!result.is_empty());
        // Verify hyphenated emotion IDs are preserved correctly
        let pair = &result[0].emotion_pair;
        assert!(
            (pair[0] == "deep-sadness" && pair[1] == "mixed-joy")
                || (pair[0] == "mixed-joy" && pair[1] == "deep-sadness"),
            "Expected hyphenated emotion IDs to be preserved, got: {:?}", pair
        );
    }

    #[test]
    fn test_compute_co_occurrence_no_related() {
        let reflections = vec![
            Reflection {
                timestamp: "2024-01-15T10:00:00Z".to_string(),
                emotion_id: Some("joy".to_string()),
                emotion_name: Some("Joy".to_string()),
                intensity: Some(7.0),
                related_emotions: None,
                location: None,
                people: None,
                coping_strategies: None,
                mood_before: None,
                mood_after: None,
            },
        ];

        let result = compute_co_occurrence(&reflections);
        // No pairs if only one emotion
        assert_eq!(result.len(), 0);
    }
}
