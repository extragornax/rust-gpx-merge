use gpx::Gpx;

pub fn merge_traces(data: &[Gpx]) -> Gpx {
    if data.is_empty() {
        return Gpx::default();
    } else if data.len() == 1 {
        return data[0].clone();
    }

    let (base, remaining_items) = data.split_at(1);
    let mut base = base[0].clone();
    remaining_items.iter().for_each(|item| {
        let local_tracks = &item.tracks;
        for lt in local_tracks {
            if base.tracks.is_empty() {
                base.tracks.push(lt.clone());
            } else {
                let local_segments = &lt.segments;
                for ls in local_segments {
                    base.tracks[0].segments.push(ls.clone());
                }
            }
        }
    });

    base
}
