use std::io::Cursor;

use image::{Rgb, RgbImage};

const IMPORTANT_TIMES: &[f64] = &[7.0, 18.5, 19.5];

const BACKGROUND_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
const LIGHT_AXIS_COLOR: Rgb<u8> = Rgb([127, 127, 127]);
const DARK_AXIS_COLOR: Rgb<u8> = Rgb([63, 63, 63]);
const BASE_GRAPH_LINE_COLOR: Rgb<u8> = Rgb([200, 64, 32]);

pub const GRAPH_NUM_DAYS: usize = 7;
const MAIN_LINE_THICKNESS: usize = 3;

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

    pub fn draw_line(&mut self, data: Vec<(String, f64)>){
        for y in 0..self.height{
            for x in 0..self.width{
                let current_index = (y * self.width) + x;
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

