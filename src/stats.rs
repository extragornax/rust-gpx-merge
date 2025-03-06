use gpx::Gpx;

pub fn print_stats(data: &Vec<Gpx>) {
    let files = data.len();
    let mut tracks = 0;
    let mut segments = 0;
    let mut points = 0;
    let mut earliest_date = None;
    let mut latest_date = None;

    for d in data {
        let _tracks = &d.tracks;
        tracks += _tracks.len();
        for t in _tracks {
            let _segments = &t.segments;
            segments += _segments.len();
            for s in _segments {
                points += s.points.len();
                for _point in &s.points {
                    if let Some(tt) = _point.time {
                        if earliest_date.is_none() {
                            earliest_date = Some(tt);
                        } else {
                            let tmp_date = earliest_date.unwrap();
                            if tmp_date > tt {
                                earliest_date = Some(tt);
                            }
                        }

                        if latest_date.is_none() {
                            latest_date = Some(tt);
                        } else {
                            let tmp_date = latest_date.unwrap();
                            if tmp_date < tt {
                                latest_date = Some(tt);
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "Files: {}\nTracks: {}\nSegments: {}\nPoints: {}",
        files, tracks, segments, points
    );
    match (earliest_date, latest_date) {
        (Some(e), Some(l)) => {
            println!("From {:?} to {:?}", e, l);
        }
        (Some(e), None) => {
            println!("From {:?}", e.format());
        }
        (None, Some(l)) => {
            println!("From {:?}", l.format());
        }
        (None, None) => {
            println!("None");
        }
    }
}
