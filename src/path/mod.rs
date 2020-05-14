pub mod track;

use std::{
  collections::HashMap,
  ops::{
    Add,
    Sub,
  },
};

use quicksilver::{
  geom::{Circle},
  graphics::{Color},
  lifecycle::{Window}
};

use super::{
  GRID_CELL_SIZE,
};

use self::track::{TrackPiece, Track, TURN_LEN, DIAG_LEN, STRT_LEN};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Pos(pub i32, pub i32);

impl Pos {
  pub fn to_float(&self) -> (f32, f32) {
    (self.0 as f32, self.1 as f32)
  }

  pub fn scale(&mut self, scl: f32) {
    self.0 = (self.0 as f32 * scl) as i32;
    self.1 = (self.1 as f32 * scl) as i32;
  }
}

//impl From<Point2> for Pos {
//  fn from(p: Point2) -> Self {
//    Pos(p.x as i32, p.y as i32)
//  }
//}
//
//impl Into<Point2> for Pos {
//  fn into(self) -> Point2 {
//    Point2::new(self.0 as f32, self.1 as f32)
//  }
//}

impl From<Dir> for Pos {
  fn from(p: Dir) -> Self {
    p.to_pos()
  }
}

//impl Into<Pos> for Dir {
//  fn into(self) -> Pos {
//    self.to_pos()
//  }
//}

impl Add for Pos {
  type Output = Pos;

  fn add(self, rhs: Pos) -> Pos {
    Pos(
      self.0 + rhs.0,
      self.1 + rhs.1,
    )
  }
}

impl Sub for Pos {
  type Output = Pos;

  fn sub(self, rhs: Pos) -> Pos {
    Pos(
      self.0 - rhs.0,
      self.1 - rhs.1,
    )
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Dir {
  Up,
  UpRight,
  Right,
  DownRight,
  Down,
  DownLeft,
  Left,
  UpLeft,
}

impl Dir {
  pub fn to_pos(&self) -> Pos {
    use self::Dir::*;

    let diag = (2f32.sqrt() * GRID_CELL_SIZE as f32) as i32;
    let strt = GRID_CELL_SIZE as i32;

    match &self {
      Up => Pos(0, strt),
      UpRight => Pos(diag, diag),
      Right => Pos(strt, 0),
      DownRight => Pos(diag, -diag),
      Down => Pos(0, -strt),
      DownLeft => Pos(-diag, -diag),
      Left => Pos(-strt, 0),
      UpLeft => Pos(-diag, diag),
    }
  }

  pub fn opposite(&self) -> Dir {
    use self::Dir::*;

    match self {
      Up => Down,
      UpRight => DownLeft,
      Right => Left,
      DownRight => UpLeft,
      Down => Up,
      DownLeft => UpRight,
      Left => Right,
      UpLeft => DownRight,
    }
  }

  pub fn from_angle(angle: f32) -> Dir {
    use self::Dir::*;

    let angle = (angle % 360.).abs();

    if angle < 22.5 { return Up };
    if angle < 67.5 { return UpRight };
    if angle < 112.5 { return Right };
    if angle < 157.5 { return DownRight };
    if angle < 202.5 { return Down };
    if angle < 247.5 { return DownLeft };
    if angle < 292.5 { return Left };
    if angle < 337.5 { return UpLeft };
    if angle < 360.5 { return Up };

    unreachable!()
  }

  pub fn into_angle(self) -> f32 {
    use self::Dir::*;

    match self {
      Up => 0.0,
      UpRight => 45.0,
      Right => 90.0,
      DownRight => 135.0,
      Down => 180.0,
      DownLeft => 235.0,
      Left => 270.0,
      UpLeft => 315.0,
    }
  }

  pub fn difference(&self, other: Dir) -> f32 {
    let self_angle = self.into_angle();
    let other_angle = other.into_angle();

    let t1 = (self_angle - other_angle).abs();
    let t2 = ((self_angle + 360.0) - other_angle).abs();
    let t3 = (self_angle - (other_angle + 360.0)).abs();

    (if t1 < t2 { if t1 < t3 { t1 } else { t3 } } else { if t2 < t3 { t2 } else { t3 } }) / 180.0
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Connection {
  pub pos: Pos,
  pub dir: Dir,
}

impl Connection {
  pub fn new(pos: Pos, dir: Dir) -> Self {
    Connection {
      pos,
      dir,
    }
  }

  fn gen_connections(&self) -> Vec<(Connection, i32)> {
    let start = *self;

    let gs = GRID_CELL_SIZE as f32;
    let (x, y) = start.pos.to_float();

    let is_x = x % gs == 0.;

    use self::Dir::*;

    let conn = |pos: (f32, f32), dir: Dir| {
      Connection::new(Pos(pos.0 as i32, pos.1 as i32), dir)
    };

    let turn = TURN_LEN as i32;
    let strt = STRT_LEN as i32;
    let diag = DIAG_LEN as i32;

    let conns = match start.dir {
      Right => vec![
        (conn((x + 1.5 * gs, y - 0.5 * gs), DownRight), turn),
        (conn((x + 1. * gs, y), Right), strt),
        (conn((x + 1.5 * gs, y + 0.5 * gs), UpRight), turn),
      ],
      UpRight => vec![
        (conn((x + 0.5 * gs, y + 0.5 * gs), UpRight), diag),
        if is_x {
          (conn((x + 0.5 * gs, y + 1.5 * gs), Up), turn)
        } else {
          (conn((x + 1.5 * gs, y + 0.5 * gs), Right), turn)
        },
      ],
      DownRight => vec![
        (conn((x + 0.5 * gs, y - 0.5 * gs), DownRight), diag),
        if is_x {
          (conn((x + 0.5 * gs, y - 1.5 * gs), Down), turn)
        } else {
          (conn((x + 1.5 * gs, y - 0.5 * gs), Right), turn)
        },
      ],
      Up => vec![
        (conn((x + 0.5 * gs, y + 1.5 * gs), UpRight), turn),
        (conn((x, y + 1. * gs), Up), strt),
        (conn((x - 0.5 * gs, y + 1.5 * gs), UpLeft), turn),
      ],
      Down => vec![
        (conn((x - 0.5 * gs, y - 1.5 * gs), DownLeft), turn),
        (conn((x, y - 1. * gs), Down), strt),
        (conn((x + 0.5 * gs, y - 1.5 * gs), DownRight), turn),
      ],
      Left => vec![
        (conn((x - 1.5 * gs, y + 0.5 * gs), UpLeft), turn),
        (conn((x - 1. * gs, y), Left), strt),
        (conn((x - 1.5 * gs, y - 0.5 * gs), DownLeft), turn),
      ],
      UpLeft => vec![
        (conn((x - 0.5 * gs, y + 0.5 * gs), UpLeft), diag),
        if is_x {
          (conn((x - 0.5 * gs, y + 1.5 * gs), Up), turn)
        } else {
          (conn((x - 1.5 * gs, y + 0.5 * gs), Left), turn)
        },
      ],
      DownLeft => vec![
        (conn((x - 0.5 * gs, y - 0.5 * gs), DownLeft), diag),
        if is_x {
          (conn((x - 0.5 * gs, y - 1.5 * gs), Down), turn)
        } else {
          (conn((x - 1.5 * gs, y - 0.5 * gs), Left), turn)
        },
      ],
    };

    conns
  }
}

const DEBUG: bool = true;

// grid size, not screen size
// #[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Path {
  start: Connection,
  path: Option<Vec<Track>>,
  debug: Vec<Track>,
}

impl Path {
  pub fn new(start: Pos, dir: Dir) -> Self {
    Path {
      start: Connection::new(start, dir),
      path: None,
      debug: Vec::new(),
    }
  }

  pub fn into_pieces(self) -> Option<Vec<Track>> {
    self.path
  }

  pub fn draw(&self, window: &mut Window) {
    // draw path
//    graphics::set_color(window, [0.0, 0.7, 0.2, 1.0].into())?;

    if DEBUG {
      for track in self.debug.iter() {
        track.draw(window, Color::PURPLE);
      }
    }

    if let Some(ref path) = self.path {
      for track in path.iter() {
        track.draw(window, Color::CYAN);
      }
    }

    // current pos
//    graphics::set_color(window, [1.0, 0.0, 0.0, 1.0].into())?;
    let pos = self.start.pos;

    window.draw(&Circle::new((pos.0, pos.1), 4), Color::RED);

//    graphics::circle(window, DrawMode::Fill, pos.into(), 4., 0.2)?;
  }

  fn estimate(from: &Connection, to: &Pos) -> i32 {
    let dx = (from.pos.0 - to.0) as f32;
    let dy = (from.pos.1 - to.1) as f32;
    let dy = if dy == 0.0 { 0.00001 } else { dy };

    // FIXME: fuggered
    let dir = Dir::from_angle(180. * (dx / -dy).atan() / std::f32::consts::PI);

    let pos_diff = (dx.abs() + dy.abs()) * 10.0;
    let pos_diff_abs = (dx.abs().powi(2) + dy.abs().powi(2)).sqrt() * 11.;
    let dir_diff = from.dir.difference(dir) * 0.0;

//    (pos_diff + dir_diff) as i32
    pos_diff_abs as i32
  }

  pub fn add_path(&mut self, to: Pos) {
    if DEBUG {
      self.debug.clear();
    }

    let path = self.find_path(to);

    self.path = match path {
      Some(path) => {
        Some(path.windows(2).map(|c| Track::from((c[0], c[1]))).collect::<Vec<Track>>())
      }
      None => None,
    };
  }

  pub fn find_path(&mut self, to: Pos) -> Option<Vec<Connection>> {
    let mut open: Vec<usize> = Vec::new();
    let mut closed: Vec<usize> = Vec::new();

    let mut nodes: Vec<Node> = Vec::new();

    let mut lookup: HashMap<Connection, usize> = HashMap::new();

    let mut children: Vec<usize> = Vec::new();

    let head = self.start;

    let start = Node {
      conn: head,
      g_score: 0i32,
      f_score: Path::estimate(&head, &to),
    };

    let mut count = 0;

    lookup.insert(start.conn, count);
    open.push(count);
    nodes.push(start);
    children.push(0);
    count += 1;

    while open.len() > 0 {
      let target: usize = open.iter().fold((0, i32::max_value()), |acc, i| {
        let node = nodes.get(*i).expect("nodes in the open list exist");
        if node.f_score < acc.1 { (*i, node.f_score) } else { acc }
      }).0;

      let node = nodes[target];

      if node.conn.pos == to {
        let mut target = target;

        let mut total = Vec::new();

        while target != 0 {
          total.push(target);
          target = children[target];
        }

        total.push(target);

        total.reverse();

        return Some(
          total
              .iter()
              .map(|i| nodes.get(*i).expect("all nodes in the children list should exist").conn)
              .collect()
        );
      }

      open.remove_item(&target);
      closed.push(target);

      for (conn, len) in node.conn.gen_connections() {
        let total_g = node.g_score + len * 10;

        if let Some(i) = lookup.get(&conn) {
          if !closed.contains(i) {
            let mut n_node = nodes.get_mut(*i).expect("nodes should exists if they are in lookup");

            if n_node.g_score <= total_g {
              continue;
            }

            let child = children.get_mut(*i).expect("a children entry should exist for all nodes");
            *child = target;

            n_node.g_score = total_g;
            n_node.f_score = total_g + Path::estimate(&n_node.conn, &to)
          }
          continue;
        }

        let n_node = Node {
          conn,
          g_score: total_g,
          f_score: total_g + Path::estimate(&conn, &to),
        };

        if DEBUG {
          self.debug.push(Track::from((node.conn, conn)));
        }

        lookup.insert(conn, count);
        open.push(count);
        nodes.push(n_node);
        children.push(target);
        count += 1;
      }
    }

    None
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
struct Node {
  conn: Connection,
  g_score: i32,
  f_score: i32,
}
