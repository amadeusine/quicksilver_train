use std::collections::VecDeque;

use rand::{Rng, thread_rng};

use quicksilver::{
  geom::{
    Circle
  },
  graphics::{Color},
  lifecycle::{Window}
};

//use ggez::{
//  Context,
//  graphics::{
//    self,
//    Point2,
////    Color,
//  },
//  GameResult,
//  timer::{
//    get_delta,
//    duration_to_f64,
//  },
//};

use super::{
  ConnectionMap,
  path::{
    track::{
      Track,
      TrackPiece,
    },
    Connection,
  },
  draw_line
};

//type Queue = VecDeque<usize>;

pub struct Train {
  segments: Vec<Segment>,
  colour: Color,
}

impl Train {
  pub fn new(speed: f32, track: usize, dist: f32, (seg_n, seg_dist, seg_len): (usize, f32, f32), tracks: &Vec<Track>, conns: &ConnectionMap) -> Self {
    // random train colour
    let mut rnd = thread_rng();
    let colour: Color = Color {
      r: rnd.gen_range(0.0, 1.0),
      g: rnd.gen_range(0.0, 1.0),
      b: rnd.gen_range(0.0, 1.0),
      a: 1.0,
    };

    // train
    let mut segments = Vec::new();

    let total_len = seg_len * seg_n as f32 + seg_dist * seg_n as f32 - 1.;
    let mut last = dist + total_len;

    // pre calculate the queue on spawn
    let mut head = Segment::new(speed, track, last);
    let delta = last / speed;
    let queue = head.update(tracks, conns, delta);
    last -= seg_len;

    segments.push(head);

    for i in 0..(seg_n * 2 - 1) {
      let mut seg = Segment::new(speed, track, last);
      for conn in queue.iter() {
        seg.push_conn(*conn);
      }
      seg.update(tracks, conns, last / speed);
      segments.push(seg);

      if i % 2 == 0 {
        last -= seg_dist;
      } else {
        last -= seg_len;
      }
    }

    Train {
      segments,
      colour,
    }
  }

  pub fn update(&mut self, _window: &mut Window, tracks: &Vec<Track>, conns: &ConnectionMap) {
    let delta = 1.0 / 60.0;

    let mut iter = self.segments.iter_mut();

    let head = iter.next().expect("Segments should always be at least 2 long, so the first element should exist");
    let queue = head.update(tracks, conns, delta);

    for seg in iter {
      for conn in queue.iter() {
        seg.push_conn(*conn);
      }
      seg.update(tracks, conns, delta);
    }
  }

  pub fn draw(&mut self, window: &mut Window) {
    for seg in self.segments.iter_mut() {
      seg.draw(window, self.colour);
    }

    for conn in self.segments.chunks(2) {
      let (start, end) = (conn[0].pos, conn[1].pos);

      draw_line(window, start.0, start.1, end.0, end.1, 10., self.colour);
    }
  }
}

pub struct Segment {
  speed: f32,
  track: usize,
  dist: f32,
  pos: (f32, f32),
  dir: i8,
  turns: VecDeque<usize>,
}

impl Segment {
  pub fn new(speed: f32, track: usize, dist: f32) -> Self {
    Segment {
      speed,
      track,
      dist,
      pos: (0., 0.),
      dir: 1,
      turns: VecDeque::new(),
    }
  }

  pub fn push_conn(&mut self, conn: usize) {
    self.turns.push_back(conn);
  }

  pub fn update_track<'a>(&mut self, tracks: &'a Vec<Track>, conns: &ConnectionMap, queue: &mut VecDeque<usize>, conn: &Connection) -> Option<&'a Track> {
    let mut rnd = thread_rng();

    if let Some(conns) = conns.get(conn) {
      let (trc, dir) = if let Some(conn) = self.turns.pop_front() {
        conns.get(conn).expect("Connections should have the element in the queue")
      } else {
        let index = if conns.len() > 1 { rnd.gen_range(0, conns.len()) } else { 0 };
        queue.push_back(index);
        conns.get(index).expect("Connections should have at least one element")
      };

      let track = tracks.get(*trc).expect("Tracks in the hashmap should exist");

      if self.dir != *dir {
        self.dir = -self.dir;
      }

      if self.dir == -1 {
        self.dist = track.len() - self.dist;
      }

      self.track = *trc;

      Some(track)
    } else {
      None
    }
  }

  pub fn use_next_track<'a>(&mut self, tracks: &'a Vec<Track>, conns: &ConnectionMap, queue: &mut VecDeque<usize>, track: &Track) -> &'a Track {
    let len = track.len();

    if self.dist > len {
      // end of track
      self.dist = self.dist - len;

      let conn = &track.end();

      let track = match self.update_track(tracks, conns, queue, conn) {
        Some(track) => {
          track
        }
        None => {
          self.dist = len - self.dist;
          self.dir = -self.dir;
          tracks.get(self.track).expect("Current track should always exist")
        }
      };

      track
    } else if self.dist < 0. {
      // start of track
      self.dist = -self.dist;

      let mut conn = track.start();
      conn.dir = conn.dir.opposite();

      let track = match self.update_track(tracks, conns, queue, &conn) {
        Some(track) => {
          track
        }
        None => {
          self.dir = -self.dir;
          tracks.get(self.track).expect("Current track should always exist")
        }
      };

      track
    } else {
      tracks.get(self.track).expect("Current track should always exist")
    }
  }

  pub fn update(&mut self, tracks: &Vec<Track>, conns: &ConnectionMap, delta: f32) -> VecDeque<usize> {
    let mut queue = VecDeque::new();

    self.dist += self.speed * delta * self.dir as f32;

    let mut track = tracks.get(self.track).expect("tracks should have the current one");
    let mut len = track.len();

    while self.dist > len || self.dist < 0. {
      track = self.use_next_track(tracks, conns, &mut queue, track);
      len = track.len();
    }

    let perc = self.dist / len;
    let pos = track.lerp(perc);

    self.pos = pos.to_float();

    queue
  }

  pub fn draw(&mut self, window: &mut Window, color: Color) {
    window.draw(&Circle::new((self.pos.0, self.pos.1), 5), color);
  }
}
