use crate::signal;
use anyhow::Result;
use futures::FutureExt;
use log::*;
use rtc::interceptor::Registry;
use rtc::media_stream::MediaStreamTrack;
use rtc::peer_connection::configuration::interceptor_registry::register_default_interceptors;
use rtc::peer_connection::configuration::media_engine::{MediaEngine, MIME_TYPE_H264};
use rtc::peer_connection::configuration::RTCConfigurationBuilder;
use rtc::peer_connection::sdp::RTCSessionDescription;
use rtc::peer_connection::transport::RTCIceServer;
use rtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use rtc::rtp;
use rtc::rtp_transceiver::rtp_sender::{
    RTCRtpCodec, RTCRtpCodecParameters, RTCRtpCodingParameters, RTCRtpEncodingParameters,
    RtpCodecKind,
};
use rtc::rtp_transceiver::{RTCRtpTransceiverDirection, RTCRtpTransceiverInit};
use std::sync::Arc;
use std::{fs::OpenOptions, io::Write, str::FromStr};
use webrtc::error::Result as WebRtcResult;
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;
use webrtc::media_stream::track_local::static_sample::TrackLocalStaticSample;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};
use webrtc::peer_connection::{
    PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler, RTCIceGatheringState,
    RTCPeerConnectionState,
};
use webrtc::runtime::{
    block_on, broadcast_channel, channel, default_runtime, sleep, BroadcastSender, Runtime, Sender,
};

// ── Event handler ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Handler {
    gather_complete_tx: Sender<()>,
    done_tx: Sender<()>,
    connected_tx: Sender<()>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Handler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.gather_complete_tx.try_send(());
        }
    }

    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        println!("Peer Connection State has changed: {state}");
        match state {
            RTCPeerConnectionState::Connected => {
                let _ = self.connected_tx.try_send(());
            }
            RTCPeerConnectionState::Failed
            | RTCPeerConnectionState::Disconnected
            | RTCPeerConnectionState::Closed => {
                let _ = self.done_tx.try_send(());
            }
            _ => {}
        }
    }
}

#[tokio::main]
pub async fn server() -> Result<()> {
    info!("starting...");

    let device_path = Path::new("/dev/video0");
    let max_fps = 60;

    let mut device = h264_webcam_stream::get_device(&device_path)?;
    let mut stream = h264_webcam_stream::stream(&mut device, max_fps)?;

    // // Everything below is the WebRTC-rs API! Thanks for using it ❤️.
    //
    // // Create a MediaEngine object to configure the supported codec
    // let mut media_engine = MediaEngine::default();
    //
    // let video_codec = RTCRtpCodecParameters {
    //     rtp_codec: RTCRtpCodec {
    //         mime_type: MIME_TYPE_H264.to_owned(),
    //         clock_rate: 90000,
    //         channels: 0,
    //         sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
    //             .to_owned(),
    //         rtcp_feedback: vec![],
    //     },
    //     payload_type: 102,
    //     ..Default::default()
    // };
    //
    // // if video_file.is_some() {
    // //     media_engine.register_codec(video_codec.clone(), RtpCodecKind::Video)?;
    // // }
    // media_engine.register_codec(video_codec.clone(), RtpCodecKind::Video)?;
    //
    // let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;
    //
    // // Create RTC peer connection configuration
    // let config = RTCConfigurationBuilder::new()
    //     .with_ice_servers(vec![RTCIceServer {
    //         urls: vec!["stun:stun.l.google.com:19302".to_string()],
    //         ..Default::default()
    //     }])
    //     .build();
    //
    // let (done_tx, mut done_rx) = channel::<()>(1);
    // let (gather_complete_tx, mut gather_complete_rx) = channel::<()>(1);
    // let (connected_tx, mut connected_rx) = channel::<()>(1);
    // let (ctrlc_tx, mut ctrlc_rx) = channel::<()>(1);
    // ctrlc::set_handler(move || {
    //     let _ = ctrlc_tx.try_send(());
    // })?;
    //
    // let runtime =
    //     default_runtime().ok_or_else(|| std::io::Error::other("no async runtime found"))?;
    //
    // let handler = Arc::new(Handler {
    //     gather_complete_tx,
    //     done_tx: done_tx.clone(),
    //     connected_tx,
    // });
    //
    // let peer_connection = PeerConnectionBuilder::new()
    //     .with_configuration(config)
    //     .with_media_engine(media_engine)
    //     .with_interceptor_registry(registry)
    //     .with_handler(handler)
    //     .with_runtime(runtime.clone())
    //     .with_udp_addrs(vec![format!("{}:0", signal::get_local_ip())])
    //     .build()
    //     .await?;
    //
    // // Add video track
    // let video_track: Option<Arc<TrackLocalStaticSample>> = {
    //     let ssrc = rand::random::<u32>();
    //     let track: Arc<TrackLocalStaticSample> =
    //         Arc::new(TrackLocalStaticSample::new(MediaStreamTrack::new(
    //             format!("webrtc-rs-stream-id-{}", RtpCodecKind::Video),
    //             format!("webrtc-rs-track-id-{}", RtpCodecKind::Video),
    //             format!("webrtc-rs-track-label-{}", RtpCodecKind::Video),
    //             RtpCodecKind::Video,
    //             vec![RTCRtpEncodingParameters {
    //                 rtp_coding_parameters: RTCRtpCodingParameters {
    //                     ssrc: Some(ssrc),
    //                     ..Default::default()
    //                 },
    //                 codec: video_codec.rtp_codec.clone(),
    //                 ..Default::default()
    //             }],
    //         ))?);
    //     peer_connection
    //         .add_track(Arc::clone(&track) as Arc<dyn TrackLocal>)
    //         .await?;
    //     Some(track)
    //     // } else {
    //     //     None
    // };
    Ok(())
}
