import {
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  PauseIcon,
  PlayIcon,
  SpeakerLoudIcon,
  SpeakerModerateIcon,
  SpeakerOffIcon,
  SpeakerQuietIcon,
} from '@radix-ui/react-icons';
import { Component, createRef } from 'react';
import ReactPlayer from 'react-player';
import screenfull from 'screenfull';
import { Slider } from '../../components/ui/slider';
import { cn, setFullscreen } from '@/lib/utils';

type MoviePlayerProps = {
  url: string;
  onDismount?: (arg0: MoviePlayerState) => void;
};

type MoviePlayerState = {
  playing: boolean;
  seeking: boolean;
  played: number;
  duration: number;
  loop: boolean;
  fullscreen: boolean;
  volume: number;
  muted: boolean;
  controlsVisible: boolean; // Track visibility of controls
  mouseInactive: boolean; // Track if mouse is inactive
};

const ICON_STYLE = 'w-7 h-7';

// Helper function to format time (seconds) into hh:mm:ss
function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = Math.floor(seconds % 60);

  let formattedTime = '';

  // Always show hours, minutes, and seconds
  if (hours > 0) {
    formattedTime += `${hours}:`;
    formattedTime += `${minutes.toString().padStart(2, '0')}:${remainingSeconds
      .toString()
      .padStart(2, '0')}`;
  } else if (minutes > 0) {
    formattedTime += `${minutes}:${remainingSeconds
      .toString()
      .padStart(2, '0')}`;
  } else {
    formattedTime += `0:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  return formattedTime;
}

export default class MoviePlayer extends Component<
  MoviePlayerProps,
  MoviePlayerState
> {
  innerPlayer = createRef<ReactPlayer>();
  outerPlayerRef = createRef<HTMLDivElement>();
  inactivityTimer: NodeJS.Timeout | null = null;

  constructor(props: MoviePlayerProps) {
    super(props);
    this.state = {
      playing: false,
      seeking: false,
      played: 0,
      duration: 0,
      loop: false,
      fullscreen: false,
      volume: 0.5, // Default volume
      muted: false,
      controlsVisible: true, // Start with controls visible
      mouseInactive: false, // Mouse is not inactive initially
    };
  }

  // #region Internal handlers
  handleMouseMove = () => {
    // Reset inactivity timer on mouse movement
    if (this.inactivityTimer) {
      clearTimeout(this.inactivityTimer);
    }

    this.setState({ controlsVisible: true, mouseInactive: false });

    // Set a new timer to hide the controls after 1.5 seconds
    this.inactivityTimer = setTimeout(() => {
      this.setState({ controlsVisible: false, mouseInactive: true });
    }, 1500);
  };

  handlePlayPause = () => {
    this.setState((prevState) => ({ playing: !prevState.playing }));
  };

  handlePlay = () => {
    this.setState({ playing: true });
  };

  handlePause = () => {
    this.setState({ playing: false });
  };

  handleSeekMouseDown = () => {
    this.setState({ seeking: true });
  };

  handleSeekChange = (value: number[]) => {
    this.setState({ played: value[0] });
  };

  handleSeekMouseUp = (value: number[]) => {
    this.setState({ seeking: false });
    if (this.innerPlayer.current) {
      this.innerPlayer.current.seekTo(value[0]);
    }
  };

  handleProgress = (state: { played: number }) => {
    if (!this.state.seeking) {
      this.setState({ played: state.played });
    }
  };

  handleEnded = () => {
    this.setState((prevState) => ({ playing: prevState.loop }));
  };

  handleDuration = (duration: number) => {
    this.setState({ duration });
  };

  handleToggleFullscreen = () => {
    this.setFullscreen(!this.state.fullscreen);
  };

  handleToggleMute = () => {
    this.setState((prevState) => ({ muted: !prevState.muted }));
  };

  handleVolumeChange = (value: number[]) => {
    this.setState({ volume: value[0] });
  };

  // #endregion Internal Handlers

  setFullscreen = (fullscreenState: boolean) => {
    this.setState({ fullscreen: fullscreenState });
    if (screenfull.isEnabled) {
      if (fullscreenState) {
        setFullscreen(true);
        screenfull.request(this.outerPlayerRef.current as Element);
      } else {
        // Exit fullscreen
        setFullscreen(false);
        screenfull.exit();
      }
    }
  };

  seekTo = (amount: number, type: 'seconds' | 'fraction' = 'seconds') => {
    this.innerPlayer.current?.seekTo(amount, type);
  };

  componentWillUnmount() {
    if (typeof this.props.onDismount === 'function') {
      this.props.onDismount(this.state);
    }

    if (this.inactivityTimer) {
      clearTimeout(this.inactivityTimer);
    }
  }

  render() {
    const { played, duration, controlsVisible, mouseInactive } = this.state;
    const elapsed = formatTime(played * duration); // Calculate elapsed based on progress
    const formattedDuration = formatTime(duration); // Format total duration

    return (
      <div
        className={cn(
          'relative w-full h-full bg-black',
          mouseInactive ? 'cursor-none' : '' // Apply cursor: none when inactive
        )}
        ref={this.outerPlayerRef}
        onMouseMove={this.handleMouseMove}
      >
        <ReactPlayer
          ref={this.innerPlayer}
          url={this.props.url}
          playing={this.state.playing}
          volume={this.state.volume}
          muted={this.state.muted}
          onPlay={this.handlePlay}
          onPause={this.handlePause}
          onEnded={this.handleEnded}
          onProgress={this.handleProgress}
          onDuration={this.handleDuration}
          controls={false}
          onClick={this.handlePlayPause}
          className='z-0'
          height='100%'
          width='100%'
        />

        {/* Controls Overlay */}
        <div
          className={cn(
            'absolute bottom-0 left-0 w-full p-4 flex justify-between items-center z-10 bg-gradient-to-t from-black via-transparent to-transparent',
            `opacity-${
              controlsVisible ? 100 : 0
            } transition-opacity duration-200` // Control visibility
          )}
        >
          {/* Play/Pause button */}
          <button onClick={this.handlePlayPause} className='text-white'>
            {this.state.playing ? (
              <PauseIcon className={ICON_STYLE} />
            ) : (
              <PlayIcon className={ICON_STYLE} />
            )}
          </button>

          <div
            className='select-none font-extralight'
            style={{ fontSize: '1.02rem' }}
          >
            {elapsed} / {formattedDuration}
          </div>

          {/* Seek Slider */}
          <Slider
            min={0}
            max={1}
            step={0.001}
            value={[this.state.played]}
            onValueChange={this.handleSeekChange}
            onPointerDown={this.handleSeekMouseDown}
            onPointerUp={() => this.handleSeekMouseUp([this.state.played])}
            className='w-1/2 mx-2' // Adjust width as necessary
          />

          {/* Volume Toggle Button */}
          <div className='flex items-center space-x-2 group relative'>
            <button
              onClick={this.handleToggleMute}
              className='text-white flex-1'
            >
              {this.state.muted || this.state.volume === 0 ? (
                <SpeakerOffIcon className={ICON_STYLE} />
              ) : this.state.volume < 0.33 ? (
                <SpeakerQuietIcon className={ICON_STYLE} />
              ) : this.state.volume < 0.66 ? (
                <SpeakerModerateIcon className={ICON_STYLE} />
              ) : (
                <SpeakerLoudIcon className={ICON_STYLE} />
              )}
            </button>

            {/* Volume Slider (Hidden by Default, Display on Hover) */}
            <Slider
              className='flex-4 w-24 opacity-0 group-hover:opacity-100 transition-opacity duration-200'
              min={0}
              max={1}
              step={0.001}
              value={[this.state.volume]}
              onValueChange={this.handleVolumeChange}
            />
          </div>

          {/* Fullscreen Button */}
          <button onClick={this.handleToggleFullscreen} className='text-white'>
            {this.state.fullscreen ? (
              <ExitFullScreenIcon className={ICON_STYLE} />
            ) : (
              <EnterFullScreenIcon className={ICON_STYLE} />
            )}
          </button>
        </div>
      </div>
    );
  }
}
