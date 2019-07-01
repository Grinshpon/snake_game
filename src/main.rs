extern crate sdl2;

use std::process;
use std::time::{Duration, Instant};
use std::boxed::Box;

use rand::Rng;

use sdl2::rect::{Rect};
use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

#[derive(PartialEq)]
enum Dir {
  Up,
  Down,
  Left,
  Right
}

enum List<T> {
  Cons(T, Box<List<T>>),
  Nil
}
#[allow(non_snake_case)]
fn List<T>(x: T) -> List<T> {
  List::Cons(x, Box::new(List::Nil))
}
impl <T> List <T> {
  fn insert(&mut self, x: T) {
    match self {
      List::Nil => *self = List::Cons(x, Box::new(List::Nil)),
      List::Cons(_, next) => next.insert(x)
    }
  }
  fn last(&self) -> Option<&T> {
    match self {
      List::Cons(val,next) => {
        match **next {
          List::Nil => Some(val),
          _ => next.last(),
        }
      },
      _ => None,
    }
  }
}

fn update_tail(tail: &mut List<Rect>, map: &mut [[u8;64];64],  x: i32, y: i32) {
  match tail {
    List::Nil => {},
    List::Cons(rect, next) => {
      let (x0,y0) = (rect.x,rect.y);
      rect.x = x;
      rect.y = y;
      map[(y0/16) as usize][(x0/16) as usize] = 0;
      map[(y/16) as usize][(x/16) as usize] = 2;
      update_tail(next, map, x0,y0);
    },
  };
}

fn update_snake(snake: &mut List<Rect>, map: &mut [[u8;64];64], dir: &Dir) {
  match snake {
    List::Nil => {},
    List::Cons(rect, next) => {
      let (x0,y0) = (rect.x,rect.y);
      let (db,dn) = match dir {
        Dir::Up => (false,(-1)),
        Dir::Down => (false,1),
        Dir::Left => (true,(-1)),
        Dir::Right => (true,1),
      };
      if db {
        rect.x += 16*dn;
      } else {
        rect.y += 16*dn;
      }
      update_tail(next, map, x0,y0);
    },
  };
}

fn add_to_tail(snake: &mut List<Rect>) {
  let x = snake.last().unwrap().x;
  let y = snake.last().unwrap().y;
  snake.insert(Rect::new(x,y,16,16));
}

fn draw_snake_head(canvas: &mut Canvas<Window>,snake: &List<Rect>) {
  match snake {
    List::Nil => {},
    List::Cons(a, _) => {
      canvas.fill_rect(Some(*a)).expect("idk it dun goofed");
    },
  };
}

fn draw_obstacles(canvas: &mut Canvas<Window>, map: &[[u8; 64]; 64]) {
  for i in 0..64 {
    for j in 0..64 {
      if map[i][j] == 2 || map[i][j] == 3 {
        canvas.fill_rect(Rect::new((j*16) as i32,(i*16) as i32,16,16))
          .expect("idk it dun goofed");
      }
    }
  }
}

fn set_border(map: &mut [[u8; 64]; 64]) {
  for n in 0..64 {
    map[n][0] = 2;
    map[n][63] = 2;
    map[0][n] = 2;
    map[63][n] = 2;
  }
}

fn gen() -> usize {
  let mut rng = rand::thread_rng();
  rng.gen_range(1,64)
}

fn move_point(map: &mut [[u8;64];64]) {
  let (mut x, mut y) = (gen(),gen());
  while map[y][x] == 1 || map[y][x] == 2 || map[y][x] == 3 {
    x = gen();
    y = gen();
  }
  map[y][x] = 3;
}

fn exit() {
  println!("Game Over!");
  process::exit(1);
}

fn main() {
    //framerate init
    let frame_duration = Duration::new(0, 1_000_000_000u32 / 15);

    //sdl init
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    let window  = match video_ctx
        .window("snake_game", 1024, 1024)
        .position_centered()
        .opengl()
        .build() {
            Ok(window) => window,
            Err(err)   => panic!("failed to create window: {}", err)
        };
    let mut canvas = match window
        .into_canvas()
        .build() {
            Ok(renderer) => renderer,
            Err(err) => panic!("failed to create renderer: {}", err)
        };
    let black = sdl2::pixels::Color::RGB(0, 0, 0);
    let white = sdl2::pixels::Color::RGB(255, 255, 255);
    //game init
    let mut map = [[0u8; 64]; 64];
    set_border(&mut map);
    let mut snake = List(Rect::new(128, 128, 16, 16));
    snake.insert(Rect::new(112,128,16,16));
    snake.insert(Rect::new(96,128,16,16));
    let mut dir = Dir::Right;
    let (mut rx, mut ry) = (8,8);
    map[ry][rx] = 1;
    map[ry][rx-1]=2;
    map[ry][rx-2]=2;
    let mut events = ctx.event_pump().unwrap();
    let mut action_taken = false;
    move_point(&mut map);
    //main loop
    let mut update = || {
      let instant = Instant::now();
      //input handler
      for event in events.poll_iter() {
        if !action_taken {
          match event {
            Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                //todo: implement pause and quit
                exit();
            },
            Event::KeyDown { keycode: Some(Keycode::Left), ..} => {
              if dir != Dir::Right && dir != Dir::Left {
                dir = Dir::Left;
              }
            },
            Event::KeyDown { keycode: Some(Keycode::Right), ..} => {
              if dir != Dir::Left && dir != Dir::Right {
                dir = Dir::Right;
              }
            },
            Event::KeyDown { keycode: Some(Keycode::Up), ..} => {
              if dir != Dir::Down && dir != Dir::Up {
                dir = Dir::Up;
              }
            },
            Event::KeyDown { keycode: Some(Keycode::Down), ..} => {
              if dir != Dir::Up && dir != Dir::Down {
                dir = Dir::Down;
              }
            },
            _ => {}
          }
          action_taken = true;
        }
      }
      //update game
      map[ry][rx] = 0;
      match dir {
        Dir::Up    => ry -= 1,
        Dir::Down  => ry += 1,
        Dir::Left  => rx -= 1,
        Dir::Right => rx += 1
      };
      if map[ry][rx] == 2 {
        //todo: implement endgame and restart
        exit();
      } else if map[ry][rx] == 3 {
        move_point(&mut map);
        add_to_tail(&mut snake);
      }
      map[ry][rx] = 1;
      update_snake(&mut snake, &mut map, &dir);
      //update sdl
      canvas.clear();
      canvas.set_draw_color(white);
      draw_snake_head(&mut canvas,&snake);
      draw_obstacles(&mut canvas, &map);
      canvas.set_draw_color(black);
      canvas.present();
      action_taken = false;
      let dt = instant.elapsed();
      if dt < frame_duration {
        std::thread::sleep(frame_duration-dt);
      }
    };
    loop {update();}
}
