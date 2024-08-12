use chrono::{DateTime, Datelike, Local, TimeZone};
use image::{self, Pixel, RgbImage};
use crate::data::DatosConFecha;

const DEBUG_COLOR: &[u8; 3] = &[255, 0, 255];
const LIGHT_LINE_COLOR: &[u8; 3] = &[170, 170, 170];
const DARK_LINE_COLOR: &[u8; 3] = &[85, 85, 85];
const LIGHT_POINT_COLOR: &[u8; 3] = &[160, 160, 200];
const DARK_POINT_COLOR: &[u8; 3] = &[50, 50, 210];

#[derive(Debug)]
struct Datapoint
{
    x: i64,
    y: i64,
}

impl Datapoint {
    fn build(data: DatosConFecha, x_start: i64, x_end: i64, y_start: f64, y_end: f64) -> Self
    {
        let outputval = Datapoint{
            x: (1080*(data.timestamp - x_start)/(x_end - x_start)) as i64,
            y: {
                if y_start == y_end
                {
                    1080/2
                }
                else
                {
                    (1080.0 - 1080.0*(data.datos - y_start)/(y_end - y_start)) as i64
                }
            },
        };
        return  outputval;
    }
}

pub fn basic_graph_builder(data: Vec<DatosConFecha>) -> RgbImage{
    let end_date = DateTime::from_timestamp(data.last().unwrap().timestamp, 0).unwrap();
    let end_timestamp = Local.with_ymd_and_hms(end_date.year(), end_date.month(), end_date.day(), 23, 59, 59).unwrap().timestamp();
    let start_date = DateTime::from_timestamp(data[0].timestamp, 0).unwrap();
    let start_timestamp = i64::min(Local.with_ymd_and_hms(start_date.year(), start_date.month(), start_date.day(), 0, 0, 0).unwrap().timestamp(), end_timestamp - 7*24*3600);
    let (min_val, max_val) = get_min_max(&data);
    let delta = max_val - min_val;

    let data: Vec<Datapoint> = data.into_iter().map(|dato| Datapoint::build(dato, start_timestamp, end_timestamp, min_val - delta*0.05, max_val + delta*0.05)).collect();

    let mut graph = basic_graph_canvas(1080, 1080);
    graph = draw_horizontal_lines(graph, min_val, delta*1.1);
    graph = draw_linear_interpolation(graph, &data, 3);
    graph = draw_points(graph, &data, 5);

    return graph;
}

fn draw_horizontal_lines(mut canvas: RgbImage, min_val: f64, delta: f64) -> RgbImage
{
    let width = canvas.width() as i64;

    let number_of_horizontal_lines = f64::ceil(delta) as i64;
    let line_separation = 1080.0/delta;
    let pad_offset = (line_separation * delta * 0.04545) as i64;
    let number_offset = (line_separation*((f64::ceil(min_val) - min_val)/delta)) as i64;
    let first_line_spacing =  pad_offset + number_offset;

    for i in 0..number_of_horizontal_lines
    {
        for x in 0..width
        {   
            let y = 1079 - (i*line_separation as i64 + first_line_spacing);
            if y < 0
            {
                return canvas;
            }
            canvas.put_pixel(x as u32, y as u32, *Pixel::from_slice(LIGHT_LINE_COLOR));
        }
    }
    return canvas;
}

fn draw_linear_interpolation(mut canvas: RgbImage, data: &Vec<Datapoint>, line_width: i64) -> RgbImage
{
    let width = canvas.width() as i64;

    for data_index in 0..(data.len() - 1)
    {
        for x in 0..width
        {
            if x < data[data_index].x || x > data[data_index + 1].x
            {
                continue;
            }

            let pendiente = (data[data_index + 1].y - data[data_index].y) as f64 / (data[data_index + 1].x - data[data_index].x) as f64;
            let line_y = (pendiente * (x - data[data_index].x) as f64) as i64 + data[data_index].y;
            //println!("pendiente: {pendiente}, line_y: {line_y}, data_i: {}, data_i+1: {}", data[data_index].y, data[data_index + 1].y);
            for y in (line_y - line_width/2)..(line_y + line_width/2)
            {
                canvas.put_pixel(x as u32, y as u32, *Pixel::from_slice(DARK_POINT_COLOR));
            }
        }
    }
    return canvas;
}

fn draw_points(mut canvas: RgbImage, data: &Vec<Datapoint>, point_radius: i64) -> RgbImage
{
    for data_point in data
    {
        for x in (i64::max(data_point.x - point_radius, 0))..(i64::min(data_point.x + point_radius, 1079))
        {
            for y in (data_point.y - point_radius)..(data_point.y + point_radius)
            {
                let diff_x = x - data_point.x;
                let diff_y = y - data_point.y;
                let distance = (diff_x*diff_x) + (diff_y*diff_y);
                if distance < (point_radius*point_radius){
                    canvas.put_pixel(x as u32, y as u32, *Pixel::from_slice(DARK_POINT_COLOR))
                }

                if distance < ((point_radius-1)*(point_radius-1)){
                    canvas.put_pixel(x as u32, y as u32, *Pixel::from_slice(LIGHT_POINT_COLOR))
                }
            }
        }
    }

    return canvas;
}

fn get_min_max(data: &Vec<DatosConFecha>) -> (f64, f64)
{
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    let mut min_timestamp = i64::MAX;
    let mut max_timestamp = i64::MIN;
    for value in data
    {
        if value.datos < min_val
        {
            min_val = value.datos;
        }
        if value.datos > max_val
        {
            max_val = value.datos;
        }
        if value.timestamp < min_timestamp
        {
            min_timestamp = value.timestamp;
        }
        if value.timestamp > max_timestamp
        {
            max_timestamp = value.timestamp;
        }
    };
    return (min_val, max_val);
}

fn basic_graph_canvas(width: u32, height: u32) -> RgbImage{
    let mut image_vec: Vec<u8> = vec![255; (width*height*3).try_into().unwrap()];
    let segment_separation = width/7;

    for y in 0..height{
        for x in 0..width{
            let primary_line_center = x%segment_separation;

            if primary_line_center < 3
            {   
                image_vec[(x*3 + 0 + width*3*y) as usize] = DARK_LINE_COLOR[0]; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = DARK_LINE_COLOR[0]; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = DARK_LINE_COLOR[0];
            }

            if primary_line_center == 10*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = LIGHT_LINE_COLOR[0]; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = LIGHT_LINE_COLOR[1]; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = LIGHT_LINE_COLOR[2];
            }

            if primary_line_center == (33*segment_separation)/48{

                image_vec[(x*3 + 0 + width*3*y) as usize] = LIGHT_LINE_COLOR[0]; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = LIGHT_LINE_COLOR[1]; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = LIGHT_LINE_COLOR[2];
            }

            if primary_line_center == 18*(segment_separation/24){
                image_vec[(x*3 + 0 + width*3*y) as usize] = LIGHT_LINE_COLOR[0]; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = LIGHT_LINE_COLOR[1]; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = LIGHT_LINE_COLOR[2]; 
            }

            if primary_line_center == 20*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = LIGHT_LINE_COLOR[0]; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = LIGHT_LINE_COLOR[1]; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = LIGHT_LINE_COLOR[2];
            }
        }
    }
    return RgbImage::from_raw(width, height, image_vec).unwrap();
}
