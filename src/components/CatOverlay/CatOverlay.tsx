import { useEffect, useRef, useState } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import catVideo from "../../assets/video/cat_listening.webm";
import "./CatOverlay.css";

/** Overlay visual mode */
type OverlayMode = "hidden" | "recording";

/** Payload for overlay_mode event from main window */
interface OverlayModePayload {
  mode: OverlayMode;
}

export function CatOverlay() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [mode, setMode] = useState<OverlayMode>("recording");

  useEffect(() => {
    if (videoRef.current) {
      // play() may return undefined in test environments
      const playPromise = videoRef.current.play();
      if (playPromise) {
        playPromise.catch((e) => console.error("[heycat] Video play failed:", e));
      }
    }
  }, []);

  // Listen for mode updates from main window
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<OverlayModePayload>("overlay_mode", (event) => {
        setMode(event.payload.mode);
      });
    };

    setupListener();

    return () => {
      unlisten?.();
    };
  }, []);

  // Build CSS class names based on state
  const containerClasses = ["cat-overlay", `cat-overlay--${mode}`]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={containerClasses}>
      <video
        ref={videoRef}
        className="cat-overlay__video"
        src={catVideo}
        loop
        muted
        playsInline
        autoPlay
      />
    </div>
  );
}
