//pub mod find_tracks;

#![feature(duration_as_u128)]
use std::time::{Duration, Instant};

use std::collections::BTreeMap;
use std::ops::Bound::{Included, Unbounded};

#[derive(Debug)]
pub struct Track {
    start: u64,
    end: u64,
    // @TODO: should this be generic?
    content: String,
}
struct Finder {
    start_map: BTreeMap<u64, Vec<usize>>,
    pub tracks: Vec<Track>,
}
enum PointType { Start, End }
struct Point {
    at: u64,
    t: PointType,
    idx: usize,
}
impl Finder {
    fn from_tracks(tracks: Vec<Track>) -> Finder {
        // The other option is to get all the points, and then for each track, iterate from
        // start_map at start, through all points < end, and push the track idx into their Vec
        let mut points = Vec::with_capacity(tracks.len() * 2);
        for (idx, track) in tracks.iter().enumerate() {
            points.push(Point{ at: track.start, t: PointType::Start, idx: idx });
            points.push(Point{ at: track.end, t: PointType::End, idx: idx });
        }
        points.sort_by(|a, b| a.at.cmp(&b.at));
        let mut track_idxs: Vec<usize> = vec![];
        let mut start_map = BTreeMap::new();
        for p in &points {
            match &p.t {
                PointType::Start => track_idxs.push(p.idx),
                // @TODO remove might be expensive
                PointType::End => track_idxs.retain(|&x| x != p.idx),
            }
            // overriding the value in the map should be fine, as this is the latest state
            start_map.insert(p.at, track_idxs.clone());
        }
        Finder {
            start_map: start_map,
            tracks: tracks,
        }
    }
    fn find(&self, x: u64) -> Option<&Vec<usize>> {
        self.start_map.range((Unbounded, Included(x)))
            .last()
            .map(|(_, idxs)| idxs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_in_range() {
        let finder = Finder::from_tracks(vec![
            Track { start: 0, end: 16, content: "".to_string() },
            Track { start: 2, end: 4, content: "".to_string() },
            Track { start: 4, end: 8, content: "".to_string() },
            Track { start: 8, end: 10, content: "".to_string() },
            Track { start: 9, end: 11, content: "".to_string() },
            Track { start: 10, end: 12, content: "".to_string() },
            Track { start: 13, end: 15, content: "".to_string() },
        ]);
        assert_eq!(finder.find(9), Some(&vec![0, 3, 4]));
        assert_eq!(finder.find(5), Some(&vec![0, 2]));
    }
    #[test]
    fn from_file() {
        let now = Instant::now();
        let subs = vtt::parse_from_file("/home/ivo/Downloads/Now_You_See_Me.vtt").unwrap().subtitles;
        println!("{}", now.elapsed().as_micros());
        let tracks: Vec<Track> = subs.iter().map(|s| Track{
            start: time_to_ms(&s.start),
            end: time_to_ms(&s.end),
            content: s.text.to_owned(),
        }).collect();
        let finder = Finder::from_tracks(tracks);
        assert_eq!(finder.find(6690000), Some(&vec![1910]));
        println!("{}", now.elapsed().as_micros());
        //println!("subs: {:?}", tracks);
    }
    fn time_to_ms(time: &vtt::Time) -> u64 {
        ((time.hours as u64)*60*60 + (time.minutes as u64)*60 + time.seconds as u64)*1000 + time.milliseconds as u64
    }
}
