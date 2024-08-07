use chrono::{DateTime, Datelike, Local, TimeZone};
use image::{self, RgbImage};
use crate::data::DatosConFecha;

const LIGHT_POINT_COLOR: (u8, u8, u8) = (160, 160, 200);
const DARK_POINT_COLOR: (u8, u8, u8) = (50, 50, 210);

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
            y: (1080.0 - 1080.0*(data.datos - y_start)/(y_end - y_start)) as i64,
        };
        //println!("{:#?}", outputval);
        return  outputval;
    }
}

pub fn basic_graph_builder(data: Vec<DatosConFecha>) -> RgbImage{
    println!("{:#?}", data);
    let end_date = DateTime::from_timestamp(data.last().unwrap().timestamp, 0).unwrap();
    let end_timestamp = Local.with_ymd_and_hms(end_date.year(), end_date.month(), end_date.day(), 23, 59, 59).unwrap().timestamp();
    let start_date = DateTime::from_timestamp(data[0].timestamp, 0).unwrap();
    let start_timestamp = i64::min(Local.with_ymd_and_hms(start_date.year(), start_date.month(), start_date.day(), 0, 0, 0).unwrap().timestamp(), end_timestamp - 7*24*3600);
    let (min_val, max_val) = get_min_max(&data);

    let data: Vec<Datapoint> = data.into_iter().map(|dato| Datapoint::build(dato, start_timestamp, end_timestamp, min_val - (max_val - min_val)*0.05, max_val + (max_val - min_val)*0.05)).collect();

    let mut graph = basic_graph_canvas(1080, 1080);
    graph = draw_points(graph, &data, 5);

    return graph;
}

fn draw_points(canvas: RgbImage, data: &Vec<Datapoint>, point_radius: i64) -> RgbImage
{
    let width = canvas.width() as i64;
    let height = canvas.height() as i64;
    let mut canvas = canvas.into_raw();

    println!("{:#?}", data);

    for y in 0..height
    {
        for x in 0..width
        {
            for data_point in data
            {
                let diff_x = x - data_point.x;
                let diff_y = y - data_point.y;
                let distance = (diff_x*diff_x) + (diff_y*diff_y);
                if distance < (point_radius*point_radius){
                    canvas[((x + y*width)*3 + 0) as usize] = DARK_POINT_COLOR.0;
                    canvas[((x + y*width)*3 + 1) as usize] = DARK_POINT_COLOR.1;
                    canvas[((x + y*width)*3 + 2) as usize] = DARK_POINT_COLOR.2;
                }

                if distance < ((point_radius-1)*(point_radius-1)){
                    canvas[((x + y*width)*3 + 0) as usize] = LIGHT_POINT_COLOR.0;
                    canvas[((x + y*width)*3 + 1) as usize] = LIGHT_POINT_COLOR.1;
                    canvas[((x + y*width)*3 + 2) as usize] = LIGHT_POINT_COLOR.2;
                }
            }
        }
    }

    return RgbImage::from_raw(width as u32, height as u32, canvas).unwrap();
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
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == 10*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == (33*segment_separation)/48{

                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == 18*(segment_separation/24){
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190; 
            }

            if primary_line_center == 20*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }
        }
    }
    return RgbImage::from_raw(width, height, image_vec).unwrap();
}
