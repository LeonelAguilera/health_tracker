use std::io::Cursor;

use chrono::{Datelike, TimeZone};
use image::{Rgb, RgbImage};

const IMPORTANT_TIMES: &[f64] = &[7.0, 18.5, 19.5];

const BACKGROUND_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
const LIGHT_AXIS_COLOR: Rgb<u8> = Rgb([127, 127, 127]);
const DARK_AXIS_COLOR: Rgb<u8> = Rgb([16, 16, 16]);
const BASE_GRAPH_LINE_COLOR: Rgb<u8> = Rgb([32, 64, 200]);

pub const GRAPH_NUM_DAYS: usize = 7;
const MAIN_LINE_THICKNESS: usize = 5;
const GRAPH_LINE_THICKNESS: usize = 7;
const CIRCLE_OUTER_RADIUS: usize = 9;
const CIRCLE_INNER_RADIUS: usize = 6;

pub struct Imagen{
    buffer: Vec<Rgb<u8>>,
    width: usize,
    height: usize,
    x_min: i64,
    x_max: i64,
    y_min: f64,
    y_max: f64,
}

impl Imagen {
    pub fn new_empty_graph(res_x: usize, res_y: usize, timescale: usize, y_min: f64, y_max: f64) -> Self{
        //let mut graph: Vec<Rgb<u8>> = Vec::with_capacity(res_x*res_y);
        let mut graph = vec![BACKGROUND_COLOR; res_x*res_y];

        let week_delta = res_x/timescale;

        for x in 0..timescale{
            for y in 0..res_y{
                for i in 0..MAIN_LINE_THICKNESS{
                    let index = (y * res_x) + (x*week_delta) + i;
                    graph[index] = DARK_AXIS_COLOR;
                }

                for hour in IMPORTANT_TIMES{
                    let x = x*week_delta + (((week_delta as f64) * hour/24.0) as usize);
                    let index = (y * res_x) + x;
                    graph[index] = LIGHT_AXIS_COLOR;
                }
            }
        }

        let current_time = chrono::offset::Local::now();
        let eod_timestamp = chrono::offset::Local.with_ymd_and_hms(current_time.year(), current_time.month(), current_time.day(), 23, 59, 59).unwrap().timestamp();
        let t0 = eod_timestamp - ((timescale * 24 * 3600) as i64);

        return Self{
            buffer: graph,
            width: res_x,
            height: res_y,
            x_min: t0,
            x_max: eod_timestamp,
            y_min,
            y_max,
        };
    }

    pub fn draw_line(&mut self, data: &Vec<(i64, f64)>){

        //let ymin = data.iter().map(|datapoint| datapoint.1).reduce(f64::min).unwrap().floor();
        //let ymax = data.iter().map(|datapoint| datapoint.1).reduce(f64::max).unwrap().ceil();

        let time_scale_factor = (self.x_max - self.x_min)/(self.width as i64);
        let y_scale_factor = (self.y_max - self.y_min)/(self.height as f64);

        let data: Vec<(f64, f64)> = data.iter()
            .map(|datapoint| (
                    ((datapoint.0 - self.x_min)/time_scale_factor) as f64,
                    ((self.y_max - datapoint.1)/y_scale_factor) as f64
                    )
                )
            .collect();
        
        //Dibujar líneas
        for i in 1..(data.len() - 1){
            let x0 = data[i].0;
            let y0 = data[i].1;
            let x1 = data[i + 1].0;
            let y1 = data[i + 1].1;

            let m = (y1 - y0)/(x1 - x0);
            let n = y0 - m*x0;

            let x0 = x0.max(0.0) as usize;
            let x1 = x1.max(0.0) as usize;

            for x in x0..x1{
                let y = ((x as f64)*m + n) as usize;
                let y_delta = ((GRAPH_LINE_THICKNESS as f64)/(2.0 * f64::cos(f64::atan(m)))) as usize;

                for y in (y.checked_sub(y_delta).unwrap_or(0))..y.min(self.height){
                    let index = (y * self.width) + x;
                    self.buffer[index] = BASE_GRAPH_LINE_COLOR;
                }
            }
        }
        
        //Dibujar círculos
        for datapoint in data{
            let x_min = (datapoint.0 as usize).checked_sub(CIRCLE_OUTER_RADIUS).unwrap_or(0);
            let x_max = (datapoint.0 as usize + CIRCLE_OUTER_RADIUS).min(self.width);
            let y_min = (datapoint.1 as usize).checked_sub(CIRCLE_OUTER_RADIUS).unwrap_or(0);
            let y_max = (datapoint.1 as usize + CIRCLE_OUTER_RADIUS).min(self.height);

            for y in y_min..y_max{
                for x in x_min..x_max{
                    let x2 = ((x as f64) - datapoint.0).powi(2) as usize;
                    let y2 = ((y as f64) - datapoint.1).powi(2) as usize;

                    if x2 + y2 <= CIRCLE_INNER_RADIUS.pow(2){
                        let index = (y * self.width) + x;
                        self.buffer[index] = BACKGROUND_COLOR;
                    }
                    else if x2 + y2 <= CIRCLE_OUTER_RADIUS.pow(2){
                        let index = (y * self.width) + x;
                        self.buffer[index] = BASE_GRAPH_LINE_COLOR;
                    }
                }
            }
        }
    }

    pub fn draw_horizontal_lines(&mut self, data: &Vec<(i64, f64)>){
        let ymin = data.iter().map(|datapoint| datapoint.1).reduce(f64::min).unwrap().floor();
        let ymax = data.iter().map(|datapoint| datapoint.1).reduce(f64::max).unwrap().ceil();
        let mut delta = (ymax - ymin).ceil();

        if delta < 2.0{
            delta += 10.0;
        }
        let denominador = 10.0f64.powf(delta.log10().floor() - 1.0);
        let delta = delta.div_euclid(denominador) as usize;

        for i in 0..delta{
            let y = i*self.height/delta;

            for x in 0..self.height{
                let index = (y * self.width) + x;
                self.buffer[index] = LIGHT_AXIS_COLOR;
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
