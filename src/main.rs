#![feature(slice_patterns)]
#![feature(vec_remove_item)]

extern crate rand;
extern crate quicksilver;

mod path;
mod train;

use std::collections::HashMap;

use quicksilver::{
  State, run, Result,
  geom::{Rectangle, Vector, Circle, Transform},
  input::{Event, MouseButton, ButtonState},
  graphics::{Color, WindowBuilder, Window, Sprite},
};

use path::{
  track::{
    Track,
    TrackPiece,
  },
  Path,
  Dir,
  Pos,
  Connection,
};

use train::Train;

const GRID_CELL_SIZE: f32 = 32.;

type ConnectionMap = HashMap<Connection, Vec<(usize, i8)>>;

struct GameState {
  mouse_pos: Pos,
  cam_pos: Pos,
  path: Option<Path>,
  tracks: Vec<Track>,
  trains: Vec<Train>,
  connections: ConnectionMap,
}

impl GameState {
  pub fn new() -> Self {
    GameState {
      mouse_pos: Pos(0, 0),
      path: None,
      tracks: Vec::new(),
      trains: Vec::new(),
      cam_pos: Pos(0, 0),
      connections: HashMap::new(),
    }
  }
}

pub(crate) fn draw_line(window: &mut Window, x: f32, y: f32, ex: f32, ey: f32, width: f32, color: Color) {
  let is_x = x != ex;
  let diagonal = is_x && y != ey;

  let dx = ex - x;
  let dy = ey - y;

  let len = if diagonal {
    (dx.abs().powi(2) + dy.abs().powi(2)).sqrt()
  } else {
    if is_x {
      (ex - x).abs()
    } else {
      (ey - y).abs()
    }
  };

  let angle = (180. * (dx / -dy).atan() / std::f32::consts::PI) - 90.;

  let center = Vector::new(x + dx / 2.0, y + dy / 2.0);

  window.draw(&Sprite::rectangle(Rectangle::new(0., 0., len, width)).with_position(center).with_transform(Transform::rotate(angle)).with_color(color));
}

fn snap_to_grid(pos: Pos) -> Pos {
  let gs = GRID_CELL_SIZE as f32;
  let pos = (pos.0 as f32, pos.1 as f32);

  // tile offset
  let off = (pos.0 % gs, pos.1 % gs);
  // grid offset
  let (rx, ry) = (pos.0 - off.0, pos.1 - off.1);
  // relative offset
  let (x, y) = (off.0 / gs, off.1 / gs);

  let res = match (x > y, x + y < 1.) {
    (true, true) => (rx + gs / 2., ry),
    (true, false) => (rx + gs, ry + gs / 2.),
    (false, true) => (rx, ry + gs / 2.),
    (false, false) => (rx + gs / 2., ry + gs),
  };

  Pos(res.0 as i32, res.1 as i32)
}

impl State for GameState {
  fn new() -> Result<Self> where Self: Sized {
    Ok(GameState::new())
  }

  fn update(&mut self, window: &mut Window) -> Result<()> {
    for train in self.trains.iter_mut() {
      train.update(window, &self.tracks, &self.connections);
    }

    Ok(())
  }

  fn event(&mut self, evt: &Event, window: &mut Window) -> Result<()> {
    let Pos(x, y) = self.mouse_pos;
    let Vector { x: mx, y: my } = window.mouse().pos();

    match evt {
      Event::MouseMoved(Vector { x, y }) => {
//        if state.middle() {
//          self.cam_pos.0 += dx;
//          self.cam_pos.1 += dy;
//        }

        let Pos(cx, cy) = self.cam_pos;

        let snap = snap_to_grid(Pos(*x as i32 + cx, *y as i32 + cy));

        if snap == self.mouse_pos {
          return Ok(());
        }

        self.mouse_pos = snap;

        if let Some(ref mut path) = self.path {
          path.add_path(snap);
        }
      }
      Event::MouseButton(button, state) => {
        match state {
          ButtonState::Pressed => {}
          _ => return Ok(()),
        }
        match button {
          MouseButton::Left => {
            if self.path.is_none() {
              let is_x = x % GRID_CELL_SIZE as i32 == 0;
              self.path = Some(Path::new(Pos(x, y), if is_x {
                if mx as i32 > x { Dir::Right } else { Dir::Left }
              } else {
                if my as i32 > y { Dir::Up } else { Dir::Down }
              }));
              return Ok(());
            }

            let mut path = None;
            std::mem::swap(&mut self.path, &mut path);
            let mut path = path.expect("we checked for none");

            if let Some(mut pieces) = path.into_pieces() {
              for track in pieces {
                let sta = track.start();
                let mut end = track.end();
                end.dir = end.dir.opposite();

                let i = self.tracks.len();

                self.connections.entry(sta).or_insert(Vec::new()).push((i, 1));
                self.connections.entry(end).or_insert(Vec::new()).push((i, -1));

                self.tracks.push(track)
              }
            }
          }
          MouseButton::Right => {
            self.trains.push(Train::new(250., 0, 0., (4, 5., 20.), &self.tracks, &self.connections));
          }
          MouseButton::Middle => {}
        }
      }
      _ => {}
    }

    Ok(())
  }

  fn draw(&mut self, window: &mut Window) -> Result<()> {
    window.clear(Color::white());

    let screen_size = window.screen_size();

    for i in 0..((screen_size.x / GRID_CELL_SIZE).ceil() as i16) {
      let x: f32 = i as f32 * GRID_CELL_SIZE;
      let y: f32 = screen_size.y;

      draw_line(window, x, 0., x, y, 1., Color::black().with_alpha(0.3));
    }

    for i in 0..((screen_size.y / GRID_CELL_SIZE).ceil() as i16) {
      let x: f32 = screen_size.x;
      let y: f32 = i as f32 * GRID_CELL_SIZE;

      draw_line(window, 0., y, x, y, 1., Color::black().with_alpha(0.3));
    }

    for track in self.tracks.iter() {
      track.draw(window, Color::black());
    }

    for train in self.trains.iter_mut() {
      train.draw(window);
    }

    if let Some(ref path) = self.path {
      path.draw(window);
    }

    window.draw(&Sprite::circle(Circle::new(self.mouse_pos.0, self.mouse_pos.1, 8)).with_color(Color::purple()));

    window.present();

    Ok(())
  }
}

fn main() {
  run::<GameState>(WindowBuilder::new("Trains!", 1280, 720).with_fullscreen(true)).unwrap();
}
