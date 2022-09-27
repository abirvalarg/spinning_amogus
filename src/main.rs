use std::{time::{Instant, Duration}, f32::consts::{PI, TAU}};

use math::{Lin, Surface};
use ultraviolet::{Vec3, Vec4, Mat4};

mod term;
mod math;

// const CHARS: [char; 8] = [' ', '.', ',', ':', 'i', 'l', 'w', 'W'];
const CHARS: &[u8; 8] = b" .,:ilwW";

fn main() {
    let cam = Mat4::from_translation(Vec3::new(0., -1., -10.));
    let model = load_model("amogus.obj");
    let delay = Duration::from_secs_f32(1./ 15.);
    let light = Vec3::new(-0.4, -0.4, -1.0).normalized();
    let mut rot = 0.;
    let mut screen: Vec<Vec<u8>> = Vec::new();
    let mut depth_buf: Vec<Vec<f32>> = Vec::new();
    let mut prev_size = None;
    let mut proj = Mat4::identity();

    loop {
        let start_time = Instant::now();
        let size = term::TermSize::get();
        if prev_size.is_some() && size == prev_size.unwrap() {
            for line in &mut screen {
                for x in line {
                    *x = CHARS[0];
                }
            }
            for line in &mut depth_buf {
                for x in line {
                    *x = 1.;
                }
            }
        } else {
            screen.resize_with(size.height as usize, || {
                let mut line = Vec::new();
                line.resize(size.width as usize, CHARS[0]);
                line
            });
            depth_buf.resize_with(size.height as usize, || {
                let mut line = Vec::new();
                line.resize(size.width as usize, 1.);
                line
            });
            proj = ultraviolet::projection::perspective_gl(PI / 3., size.width as f32 / size.height as f32, 0.1, 100.);
            prev_size = Some(size);
        }

        let rot_mat = Mat4::from_rotation_y(rot);

        for triangle in &model {
            let triangle = triangle.map(|vert| proj * cam * rot_mat * vert);

            let v1: Vec3 = (triangle[1] - triangle[0]).into();
            let v2: Vec3 = (triangle[2] - triangle[0]).into();

            let n = v1.cross(v2);
            let val = (1. - light.dot(n)) / 2.;

            raster(&mut screen, &mut depth_buf, triangle, val);
        }

        for (idx, line) in screen.iter().enumerate() {
            if idx > 0 {
                term::put('\n');
            }
            term::put_line(&line[..]);
        }

        rot += PI / 30.;
        if rot > TAU {
            rot -= TAU;
        }
        let elapsed = start_time.elapsed();
        if elapsed < delay {
            let time_left = delay - elapsed;
            std::thread::sleep(time_left);
        }
    }
}

fn load_model(path: &str) -> Vec<[Vec4; 3]> {
    let src = std::fs::read_to_string(path).unwrap();
    let mut verts = Vec::new();
    let mut res = Vec::new();
    for line in src.lines() {
        let data = line.split(" ").collect::<Vec<&str>>();
        if data.len() > 0 {
            match data[0] {
                "#" => (),
                "v" => {
                    let vert = Vec4::new(
                        data[1].parse().unwrap(),
                        data[2].parse().unwrap(),
                        data[3].parse().unwrap(),
                        1.
                    );
                    verts.push(vert);
                }
                "f" => {
                    let triangles = data[1..].iter().map(|grp| grp.split("/").next().unwrap().parse().unwrap()).collect::<Vec<usize>>();
                    for start in 0..triangles.len() - 2 {
                        res.push([
                            verts[triangles[start] - 1],
                            verts[triangles[start + 1] - 1],
                            verts[triangles[start + 2] - 1]
                        ])
                    }
                }
                _ => ()
            }
        }
    }
    res
}

fn raster(screen: &mut Vec<Vec<u8>>, depth_buf: &mut Vec<Vec<f32>>, triangle: [Vec4; 3], val: f32) {
    let mut triangle = triangle.map(|x| {
        let res = x / x[3];
        <Vec4 as Into<Vec3>>::into(res)
    });
    triangle.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
    let screen_coords = triangle.map(|coords|
        [(coords[0] + 1.) / 2. * screen[0].len() as f32, (1. - coords[1]) / 2. * screen.len() as f32]
    );
    let f1 = Lin::from((screen_coords[0], screen_coords[2]));
    let f2 = Lin::from((screen_coords[0], screen_coords[1]));
    let f3 = Lin::from((screen_coords[1], screen_coords[2]));
    let surf = Surface::from(triangle);
    let val = CHARS[((val * 7.) as usize).min(7)];
    for x in (screen_coords[0][0].ceil() as usize)..(screen_coords[2][0].ceil() as usize) {
        let y1 = f1.at(x as f32);
        let y2 = if (x as f32) < screen_coords[1][0] {
            f2.at(x as f32)
        } else {
            f3.at(x as f32)
        };
        for y in (y1.min(y2).round() as usize)..(y1.max(y2).round() as usize) {
            let real_x = x as f32 / screen[0].len() as f32 * 2. - 1.;
            let real_y = 1. - (y as f32 / screen.len() as f32 * 2.);
            let z = surf.at_x_y(real_x, real_y);
            if y >= screen.len() {
                break;
            }
            if z < 1. && z < depth_buf[y][x] {
                screen[y][x] = val;
                depth_buf[y][x] = z;
            }
        }
    }
}
