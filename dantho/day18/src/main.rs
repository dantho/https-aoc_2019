/// https://adventofcode.com/2019/day/15
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";
const DBG: bool = false;
const INFINITY: usize = std::usize::MAX/1_000_000_000_000_000*1_000_000_000_000_000-1;  // Very nearly max with lots of 999's at end to be visible as a "special" number

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::{BTreeMap,HashSet};
use ExplorerMovement::*;
use MapData::*;
use Error::*;

type Location = (usize,usize);

fn main() -> Result<(),Error> {
    let filename = "ex1.txt";
    let part1 = initiate_search(filename)?;
    println!("Min Step Count to clear keys is {}", part1);
    Ok(())
}

#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
    ItemNotFound {msg: String},
}
#[derive(PartialEq, Debug)]
struct SearchPath {
    items: Vec<(char,usize,Location)>,
}
impl ToString for SearchPath {
    fn to_string(&self) -> String {
            self.items.iter().map(|(ch,_,_)|{*ch}).collect::<String>()
    }
}
impl SearchPath {
    fn new(first_key: (char,usize, Location)) -> Self {
        let items = vec![first_key];
        SearchPath {items}
    }
}
fn initiate_search(filename: &'static str) -> Result<usize,Error> {
    let mut room_map = WorldMap::new(filename)?;
    room_map.redraw_screen()?;
    room_map.clear_distances();
    let keynames: String = room_map.data.iter().filter_map(|(_,item)|{match item
    {
        Key(k,_) => Some(k),
        _ => None
    }}).collect();
    let doornames: String = room_map.data.iter().filter_map(|(_,item)|{
        match item {
            Door(d,_) => Some(d),
            _ => None
        }}).collect();
    println!("Keys: {}", keynames);
    println!("Doors: {}", doornames);
    let alleys = match room_map.find_all_items(0, room_map.find_entrance()?) {
        None => panic!("Empty room??"),
        Some(vec_of_vecs) => vec_of_vecs,
    };
    println!("Alleys:");
    for alley in &alleys {
        println!("{:?}", alley);
    }
    let orig_alleys_with_doors: HashSet<String> = alleys.iter().map(|alley|{
        alley.to_string()
    }).collect();
    println!("Alleys (as string) with doors:");
    for alley in &orig_alleys_with_doors {
        println!("{}", alley);
    }
    // build dependency tree (bush?)
    let mut key_dependencies: BTreeMap<char,Vec<char>> = BTreeMap::new();
    for alley_str in &orig_alleys_with_doors {
        let mut door_keys:Vec<char> = Vec::new();
        for dk in alley_str.chars().rev() {
            match MapData::try_from(dk)? {
                Entrance(_) => {door_keys.push('@');}, // entrance is (FIRST DOOR and) FIRST KEY
                Door(door,_) => {door_keys.push(to_key(door));}, // doors are abstractions: they create key dependancies
                Key(key,_) => {
                    // the following match loop assumes that alleys can have duplicate sections, so keys can be found multiple times, redundantly.
                    match key_dependencies.get_mut(&key) {
                        Some(deps) => for d in &door_keys {deps.push(*d);}, // this key was found previously, processing a sibling alley, append door_keys as deps in case one is new, though this is likely all redundant
                        None => {key_dependencies.insert(key, door_keys.clone());}, // this key is completely new, clone door_keys as dependants.
                    }
                },
                _ => panic!("Should be a key, door, or entrance.  No?"),
            }
        }
    }
    println!("Dependency Tree:");
    for k in &key_dependencies {
        println!("{:?}", k);
    }
    let remaining_key_dependencies = key_dependencies;
    let min_step_count = room_map.find_min_steps('@', remaining_key_dependencies, &mut BTreeMap::new())?;
    Ok(min_step_count)
}
fn reachable(deps: &BTreeMap<char,Vec<char>>)-> Vec<char> {
    deps.iter().filter_map(|(candidate, sub_deps)| {
        let mut key_has_no_dependants = true; // Optimistic default
        for dep_key in sub_deps {
            if deps.contains_key(dep_key) {
                key_has_no_dependants = false;
                break;
            }
        }
        if key_has_no_dependants {
            Some(*candidate)
        } else {
            None
        }
    }).collect()
}
fn to_key(ch: char) -> char {
    ch.to_ascii_lowercase()
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty(usize),
    Wall,
    Entrance(usize),
    Door(char,usize),
    Key(char,usize),
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Empty(_) => '.',
            Wall => '#',
            Entrance(_) => '@',
            Door(d,_) => d,
            Key(k,_) => k,
        }
    }
}
impl TryFrom<char> for MapData {
    type Error = Error;
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        use MapData::*;
        let status = match ch {
            en if en == Entrance(0).to_char() => Entrance(INFINITY),
            mt if mt == Empty(0).to_char() => Empty(INFINITY),
            w if w == Wall.to_char() => Wall,
            d if d.is_alphabetic() && d.is_uppercase() => Door(d,INFINITY),
            k if k.is_alphabetic() && k.is_lowercase() => Key(k,INFINITY),
            _ => return Err(Error::IllegalMapData { ch }),
        };
        Ok(status)
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum ExplorerMovement {
    North=1,
    South=2,
    West=3,
    East=4,
}
impl ExplorerMovement {
    fn move_from(&self, loc: Location) -> Location {
        match self {
            North => (loc.0-1, loc.1),
            South => (loc.0+1, loc.1),
            West => (loc.0, loc.1-1),
            East => (loc.0, loc.1+1),
        }
    }
}
#[derive(Debug,Clone)]
struct WorldMap {
    data: BTreeMap<Location, MapData>,
    key_pair_cache: BTreeMap<(char,char), usize>,
    key_index: BTreeMap<char,(usize,usize)>,
}
impl WorldMap {
    fn map_distance(&mut self, loc: Location, distance: usize) -> Result<(),Error> {
        let this_loc = match self.data.get_mut(&loc) {
            Some(Empty(dist)) => dist,
            Some(Entrance(dist)) => dist,
            Some(Key(_,dist)) => dist,
            Some(Door(_,dist)) => dist,
            _ => return Ok(()), // END RECURSION (Wall or unknown found)
        };
        if *this_loc <= distance {
            return Ok(()) // END RECURSION (crossed [equiv or superior] path)
        }
        *this_loc = distance; // Set present location
        // Recurse into cardinal directions
        self.map_distance((loc.0-1,loc.1), distance+1)?; // North
        self.map_distance((loc.0+1,loc.1), distance+1)?; // South
        self.map_distance((loc.0,loc.1-1), distance+1)?; // West
        self.map_distance((loc.0,loc.1+1), distance+1)?; // East
        Ok(())
    }
    // recursive memoized step count algo -- this is the key (no pun intended) to optimizing the search
    fn find_min_steps(&mut self,
            current_key: char,
            remaining_key_dependencies: BTreeMap<char,Vec<char>>,
            mut key_path_cache: &mut BTreeMap<(char, BTreeMap<char,Vec<char>>),usize>
        ) -> Result<usize,Error> {
        if 0 == remaining_key_dependencies.len() {
            return Ok(0); // End recursion
        }
        // Is best path (from here) already cached? 
        let cache_key = (current_key, remaining_key_dependencies.clone());
        if let Some(dist) = key_path_cache.get(&cache_key) {
            return Ok(*dist);
        }
        // Else recursively calculate (from here), cache, and return best path
        let mut best_path = INFINITY;
        for key in reachable(&remaining_key_dependencies) {
            let mut deps_less_key = remaining_key_dependencies.clone();
            deps_less_key.remove(&key);
            let d = self.step_count_between(current_key, key)? + self.find_min_steps(key, deps_less_key, &mut key_path_cache)?;
            if d < best_path {
                best_path = d;
            }
        }
        key_path_cache.insert(cache_key, best_path); 
        Ok(best_path)
    }
    fn find_all_items(&mut self, present_distance: usize, present_loc: Location) -> Option<Vec<SearchPath>> {
        let mut key_found_here = false;
        let mut door_found_here = false;
        let mut entrance_found_here = false;
        let mut item_name: Option<char> = None;
        match self.data.get_mut(&present_loc) {
            Some(Wall) => {return None}, // Hit a wall.  END RECURSION
            Some(Empty(dist)) => {
                if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
                *dist = present_distance; // label this location with distance marker -- critical step! Continue searching
            },
            Some(Entrance(dist)) => {
                if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
                if DBG {println!("Found {:?} at {:?}", Entrance(present_distance), present_loc );}
                item_name = Some('@');
                *dist = present_distance; // label this location with distance marker -- critical step! Continue searching
                entrance_found_here = true; // FOUND a target.  Continue searching for more.
            },
            Some(Door(d,dist)) => {
                if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
                if DBG {println!("Found {:?} at {:?}", Door(*d,present_distance), present_loc );}
                item_name = Some(*d);
                *dist = present_distance; // label this location with distance marker -- critical step! Continue searching
                door_found_here = true; // FOUND a target.  Continue searching for more.
            }, // Hit a locked door.  END RECURSION
            Some(Key(k,dist)) => {
                if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
                if DBG {println!("Found {:?} at {:?}", Key(*k,present_distance), present_loc );}
                item_name = Some(*k);
                *dist = present_distance; // label this location with distance marker -- critical step! Continue searching
                key_found_here = true; // FOUND a target.  Continue searching for more.
            },
            None => panic!{"We stumbled off the map, somehow!"},
        }
        // recurse in cardinal directions here using present_distance + 1
        let mut multiple_paths = Vec::new();
        for dir in vec![North, South, East, West] {
            if let Some(mut vec_of_paths) = self.find_all_items(present_distance+1, dir.move_from(present_loc)) {
                 multiple_paths.append(&mut vec_of_paths);
            }
        }
        if key_found_here||door_found_here||entrance_found_here {
            let this_item = (item_name.unwrap(),present_distance,present_loc);
            match multiple_paths.len() {
                0 => {
                    multiple_paths.push(SearchPath::new(this_item));
                    if DBG {println!("Sole key on this path: {:?}", multiple_paths);}
                    Some(multiple_paths)    
                },
                1 => {
                    for path in &mut multiple_paths {
                        path.items.push(this_item);
                    }
                    if DBG {println!("Key added to single path: {:?}", multiple_paths);}
                    Some(multiple_paths)    
                },
                _ => {
                    for path in &mut multiple_paths {
                        path.items.push(this_item);
                    }
                    if DBG {println!("Key added to multiple_paths: {:?}", multiple_paths);}
                    Some(multiple_paths)    
                },
            }    
        } else {
            match multiple_paths.len() {
                0 => None,
                _ => {
                    Some(multiple_paths)
                },
            }    
        }
    }
    fn step_count_between(&mut self, key_a: char, key_b: char) -> Result<usize,Error> {
        match self.key_pair_cache.get(&(key_a,key_b)) {
            Some(dist) => {
                if *dist > INFINITY - 1 {panic!(format!("Infinite result IN CACHE! @ {:?}", (key_a,key_b)))}
                Ok(*dist) // FAST results
            }, 
            None => { // SLOW results
                println!("Pair: {:?}", (key_a,key_b));
                let loc_a = *self.key_index.get(&key_a).unwrap();
                self.clear_distances();
                self.map_distance(loc_a,0)?;
                for (key_other, loc_other) in &self.key_index {
                    if *key_other!=key_a {
                        println!("pair: {:?}", (key_a, key_other));
                        let distance_between = match self.data.get(loc_other) {
                            Some(Key(_,dist)) => *dist,
                            Some(Entrance(dist)) => *dist,
                            _ => return Err(MapAssertFail {msg: "Expected Key here.".to_string()}),
                        };
                        self.key_pair_cache.insert((key_a,*key_other), distance_between);
                        self.key_pair_cache.insert((*key_other,key_a), distance_between);
                    }
                }
                Ok(match self.key_pair_cache.get(&(key_a,key_b)) {
                    Some(steps) => {
                        if *steps > INFINITY - 1 {panic!(format!("Infinite result from Dijkstra! @ {:?}", (key_a,key_b)))}
                        *steps
                    },
                    None => panic!(format!("Didn't find pair: {:?}", (key_a,key_b))),
                })
            }
        }
    }
    fn clear_distances(&mut self) {
        for (_,item) in &mut self.data {
            match item {
                Entrance(dist) => *dist = INFINITY,
                Empty(dist)    => *dist = INFINITY,
                Key(_,dist)    => *dist = INFINITY,
                Door(_,dist)   => *dist = INFINITY,
                Wall           => (),
            }
        }
    }
    fn find_entrance(&self) -> Result<Location,Error> {
        let entrance = self.data.iter().fold(None,|found, (loc,item)| {
            match *item {
                Entrance(_) => Some(loc),
                _ => found,
            }
        });
        let entrance = match entrance {
            Some(e) => *e,
            None => return Err(ItemNotFound {msg: format!("{:?}", Entrance(0))}),
        };
        Ok(entrance)
    }
    fn new(filename: &'static str) -> Result<Self,Error> {
        let data = WorldMap::read_initial_map(filename)?;
        let key_pair_cache = BTreeMap::new();
        let key_index = data.iter().filter_map(|(loc, item)| {
            match item {
                Key(k,_) => Some((*k,*loc)),
                Entrance(_) => Some(('@',*loc)),
                _ => None,
            }
        }).collect::<BTreeMap<_,_>>();    
        Ok(WorldMap {data, key_pair_cache, key_index})
    }
    fn read_initial_map(filename: &'static str) -> Result<BTreeMap<Location,MapData>,Error> {
        let mut new_world = BTreeMap::new();
        let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
        let buf = BufReader::new(fd);
        buf.lines().enumerate().for_each(|(y, line)| {
            line.unwrap().chars().enumerate().for_each(|(x,ch)| {
                let map_item: MapData = match ch.try_into() {
                    Ok(map_data) => map_data,
                    Err(e) => panic!(format!("Error inside closure: '{:?}'", e)),
                };
                if let Some(_) = new_world.insert((y,x), map_item) {
                    assert!(false, "Overwritting data while reading.  Not locsible given code design.");
                };
            });
        });
        Ok(new_world)
    }
    fn draw_location(&self, loc: Location) -> Result<(),Error> {
        set_cursor_loc(loc.0, loc.1);
        let map_item = match self.data.get(&loc) {
            None => ' ',
            Some(data) => data.to_char(),
        };
        print_char(map_item);
        Ok(())
    }
    fn redraw_screen(&self) -> Result<(),Error> {
        print(ESC_CLS); // clear screen, reset cursor
        print(ESC_CURSOR_OFF); // Turn OFF cursor
        // print(ESC_CURSOR_ON); // Turn ON cursor
        for (loc, _) in &self.data {
            self.draw_location(*loc)?;
        }
        println!("");
        Ok(())
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}
fn print_char(ch: char) {
    print!("{}",ch);
    stdout().flush().unwrap();
}
fn set_cursor_loc(y:usize,x:usize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}

#[test]
fn test_ex1() -> Result<(),Error> {
    assert_eq!(initiate_search("ex1.txt")?, 8);
    Ok(())
}
#[test]
fn test_ex2() -> Result<(),Error> {
    assert_eq!(initiate_search("ex2.txt")?, 86);
    Ok(())
}
#[test]
fn test_ex3() -> Result<(),Error> {
    assert_eq!(initiate_search("ex3.txt")?, 132);
    Ok(())
}
#[test]
fn test_ex4() -> Result<(),Error> {
    assert_eq!(initiate_search("ex4.txt")?, 136);
    Ok(())
}
#[test]
fn test_ex5() -> Result<(),Error> {
    assert_eq!(initiate_search("ex5.txt")?, 81);
    Ok(())
}
#[test]
fn test_input() -> Result<(),Error> {
    assert_eq!(initiate_search("input.txt")?, 4900);
    Ok(())
}
