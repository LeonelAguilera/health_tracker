use std::io::Cursor;

use chrono::{Datelike, TimeZone};
use image::{Rgb, RgbImage};

const IMPORTANT_TIMES: &[f64] = &[7.0, 18.5, 19.5];

const BACKGROUND_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
const LIGHT_AXIS_COLOR: Rgb<u8> = Rgb([127, 127, 127]);
const DARK_AXIS_COLOR: Rgb<u8> = Rgb([63, 63, 63]);
const BASE_GRAPH_LINE_COLOR: Rgb<u8> = Rgb([200, 64, 32]);

pub const GRAPH_NUM_DAYS: usize = 7;
const MAIN_LINE_THICKNESS: usize = 5;
const GRAPH_LINE_THICKNESS: usize = 3;

pub struct Imagen{
    buffer: Vec<Rgb<u8>>,
    width: usize,
    height: usize,
}

impl Imagen {
    pub fn new_empty_graph(res_x: usize, res_y: usize) -> Self{
        let mut graph: Vec<Rgb<u8>> = Vec::with_capacity(res_x*res_y);

        let week_delta = res_x/GRAPH_NUM_DAYS;

        for _y in 0..res_y{
            for x in 0..res_x{

                if x % week_delta < MAIN_LINE_THICKNESS{
                    graph.push(DARK_AXIS_COLOR);
                }
                else {
                    graph.push(BACKGROUND_COLOR);
                }
            }
        }

        return Self{
            buffer: graph,
            width: res_x,
            height: res_y,
        };
    }

    pub fn draw_line(&mut self, data: Vec<(i64, f64)>){
        let current_time = chrono::offset::Local::now();
        let eod_timestamp = chrono::offset::Local.with_ymd_and_hms(current_time.year(), current_time.month(), current_time.day(), 23, 59, 59).unwrap().timestamp();
        let t0 = eod_timestamp - ((GRAPH_NUM_DAYS * 24 * 3600) as i64);

        let time_scale_factor = (eod_timestamp - t0)/(self.width as i64);
        let data: Vec<(i64, f64)> = data.iter().map(|datapoint| (((datapoint.0 - t0)/time_scale_factor, datapoint.1))).collect();
        let ymin = data.iter().map(|datapoint| datapoint.1).reduce(f64::min).unwrap().floor();
        let ymax = data.iter().map(|datapoint| datapoint.1).reduce(f64::max).unwrap().ceil();
        let y_scale_factor = (ymax - ymin)/(self.height as f64);

        for i in 1..(data.len() - 1){
            println!("{data:#?}");
            let x0 = data[i].0 as f64;
            let y0 = data[i].1;
            let x1 = data[i + 1].0 as f64;
            let y1 = data[i + 1].1;

            let m = (y1 - y0)/(x1 - x0);
            let n = y0 - m*x0;

            let x0 = x0.max(0.0) as usize;
            let x1 = x1.max(0.0) as usize;


            for x in x0..x1{
                let y = ((((x as f64)*m + n) - ymin)/y_scale_factor) as usize;
                let y_delta = ((GRAPH_LINE_THICKNESS as f64)/(2.0 * f64::cos(f64::atan(m)))) as usize;

                for y in (y - y_delta)..(y + y_delta){
                    let index = (y * self.width) + x;
                    self.buffer[index] = BASE_GRAPH_LINE_COLOR;
                }
            }
        }

    }

    pub fn into_bytes(&self) -> Vec<u8>{
        let graph = self.buffer.iter()
            .map(|pix| Vec::from(pix.0))
            .flatten()
            .collect::<Vec<u8>>();
        let image = RgbImage::from_raw(self.width as u32, self.height as u32, graph).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        let _ = image.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png);

        return buffer;
    }
}

