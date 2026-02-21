use std::path::Path;
use std::sync::Mutex;

use osmpbf::Element;
use osmpbf::ElementReader;
use rayon::prelude::*;

use crate::Tree2D;

pub struct ImportedNodes {
    pub countries: Tree2D<i64, String>,
    pub settlements: Tree2D<i64, String>,
    pub other: Tree2D<i64, String>,
}

impl ImportedNodes {
    pub fn import_pbf(file: &Path) -> std::io::Result<ImportedNodes> {
        enum Kind {
            Country,
            Settlement,
            Other,
        }

        fn push_nodes<'a, 'b>(
            tags: impl IntoIterator<Item = (&'a str, &'b str)>,
            latitude: i64,
            longitude: i64,
            other: &Mutex<Vec<([i64; 2], String)>>,
            settlements: &Mutex<Vec<([i64; 2], String)>>,
            countries: &Mutex<Vec<([i64; 2], String)>>,
        ) {
            let mut name = None;
            let mut name_en = None;
            let mut kind = Kind::Other;
            for (key, value) in tags {
                match key {
                    "name" => name = Some(value),
                    "name:en" => name_en = Some(value),
                    "place"
                        if [
                            "city",
                            "town",
                            "village",
                            "hamlet",
                            "isolated_dwelling",
                            "farm",
                            "allotments",
                        ]
                        .contains(&value) =>
                    {
                        kind = Kind::Settlement
                    }
                    "place" if value == "country" => kind = Kind::Country,
                    _ => {}
                }
            }
            let Some(name) = name_en.or(name) else {
                return;
            };
            let node = ([longitude, latitude], name.to_string());
            let nodes = match kind {
                Kind::Country => countries,
                Kind::Settlement => settlements,
                Kind::Other => other,
            };
            nodes.lock().unwrap().push(node);
        }

        let reader = ElementReader::from_path(file)?;
        let other = Mutex::new(Vec::with_capacity(2 * 1024 * 1024));
        let settlements = Mutex::new(Vec::with_capacity(2 * 1024 * 1024));
        let countries = Mutex::new(Vec::with_capacity(2 * 1024 * 1024));
        reader.par_map_reduce(
            |element| match element {
                Element::Node(node) => push_nodes(
                    node.tags(),
                    node.nano_lat(),
                    node.nano_lon(),
                    &other,
                    &settlements,
                    &countries,
                ),
                Element::DenseNode(node) => push_nodes(
                    node.tags(),
                    node.nano_lat(),
                    node.nano_lon(),
                    &other,
                    &settlements,
                    &countries,
                ),
                // TODO Element::Way
                _ => {}
            },
            || (),
            |(), ()| (),
        )?;
        let mut other = other.into_inner().unwrap();
        other.par_sort_unstable();
        let mut settlements = settlements.into_inner().unwrap();
        settlements.par_sort_unstable();
        let mut countries = countries.into_inner().unwrap();
        countries.par_sort_unstable();
        Ok(Self {
            other: Tree2D::from_nodes(other),
            settlements: Tree2D::from_nodes(settlements),
            countries: Tree2D::from_nodes(countries),
        })
    }
}
