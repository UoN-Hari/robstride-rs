mod robstride;

use std::f32::consts::PI;
use tokio::time::{Duration, sleep};
use futures_util::{SinkExt, StreamExt};
use socketcan::{tokio::CanSocket, CanFrame, Result, EmbeddedFrame, Id};
use tokio;
use robstride::RobStrideUtils;
// use robstride::StopMode;


#[tokio::main]
async fn main() -> Result<()> {
    // 电机设置工具初始化
    let mut rs_util = RobStrideUtils::new(0x7f);

    // CAN Socket接口初始化
    let sock_tx = CanSocket::open("can0")?;
    let mut sock_rx = CanSocket::open("can0")?;

    // 电机初始化参数设定
    sock_tx.write_frame(rs_util.request_dev_id())?.await?;
    sock_tx.write_frame(rs_util.request_enable())?.await?;
    sock_tx.write_frame(rs_util.write_param(0x7005, 0u32))?.await?;
    sock_tx.write_frame(rs_util.write_param(0x7010, unsafe { std::mem::transmute(0.04f32) }))?.await?;
    sock_tx.write_frame(rs_util.write_param(0x7011, unsafe { std::mem::transmute(0.006f32) }))?.await?;
    sock_tx.write_frame(rs_util.write_param(0x7014, unsafe { std::mem::transmute(0.02f32) }))?.await?;

    // 反馈数据储存数组
    let mut fb_data: [f32; 4] = [0f32; 4];

    // 电机实时线程
    loop { // 控制频率 1000Hz
        sock_tx.write_frame(rs_util.request_motion(0.0, 2f32, 0f32, 0.15f32, 0.2f32))?.await?;
        match sock_rx.next().await.unwrap() {
            Ok(CanFrame::Data(frame)) => {
                if frame.id() == Id::from(rs_util.extended_id(2, 0x807f, 0x00)) {
                    let mut bytes: [u8; 2] = [0; 2];
                    bytes.copy_from_slice(&frame.data()[0..2]);
                    fb_data[0] = u16::from_be_bytes(bytes) as f32 / 65535f32 * (8f32 * PI) - (4f32 * PI);
                    bytes.copy_from_slice(&frame.data()[2..4]);
                    fb_data[1] = u16::from_be_bytes(bytes) as f32 / 65535f32 * 88f32 - 44f32;
                    bytes.copy_from_slice(&frame.data()[2..4]);
                    fb_data[2] = u16::from_be_bytes(bytes) as f32 / 65535f32 * 34f32 - 17f32;
                    bytes.copy_from_slice(&frame.data()[2..4]);
                    fb_data[3] = u16::from_be_bytes(bytes) as f32 / 1000f32;
                }
            },
            Ok(CanFrame::Remote(frame)) => println!("{:?}", frame),
            Ok(CanFrame::Error(frame)) => println!("{:?}", frame),
            Err(err) => eprintln!("{}", err),
        }
        println!("Motor status: Ang: {:<10}, Vel: {:<15}, Torque: {:<15}, Temperature: {:<15}", fb_data[0], fb_data[1], fb_data[2], fb_data[3]);
        sleep(Duration::from_millis(1)).await;
    }
    Ok(())
}
