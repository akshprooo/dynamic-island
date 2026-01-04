const islandContainer = document.querySelector("#island");
const contentContainer = document.querySelector('.content');

let isIdle = true;
let isExpanded = false;
let isAnimating = false;
let current_song = { title: "", artist: "", cover_art: "" };
let clockInterval = null;

gsap.from(islandContainer, {
  opacity: 0,
});

function updateClock() {
  if (!isIdle) return;
  
  const now = new Date();
  const dateStr = now.toLocaleDateString('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric'
  });
  const timeStr = now.toLocaleTimeString('en-IN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: true
  });
  
  contentContainer.innerHTML = `
    <div class="idle basics">
      <div class="date">${dateStr}</div>
      <div class="time">${timeStr}</div>
    </div>`;
}

function startIdleClock() {
  if (clockInterval) {
    clearInterval(clockInterval);
  }
  
  isIdle = true;
  updateClock(); // Update immediately
  clockInterval = setInterval(updateClock, 1000);
  
  gsap.to(islandContainer, {
    width: "200px",
    ease: 'power2.inOut',
    duration: 0.8
  });
}

function stopIdleClock() {
  if (clockInterval) {
    clearInterval(clockInterval);
    clockInterval = null;
  }
  isIdle = false;
}

function expand_pill() {
  if (isExpanded || isAnimating) return;
  contentContainer.innerHTML=''
  isAnimating = true;
  isExpanded = true;
  stopIdleClock();
  
  gsap.to(islandContainer, {
    width: "300px",
    ease: 'elastic.out(.5, 0.3)',
    duration: 1.4,
    onComplete: () => {
      isAnimating = false;
    }
  });
}

function collapse_pill() {
  if (!isExpanded || isAnimating) return;
  contentContainer.innerHTML=''
  isAnimating = true;
  isExpanded = false;
  gsap.to(islandContainer, {
    width: '200px',
    ease: 'elastic.out(.1, 0.7)',
    duration: 1.4,
    onComplete: () => {
      current_song.title = "";
      current_song.artist = "";
      current_song.cover_art = "";
      isAnimating = false;
      startIdleClock();
    }
  });
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function hasTrackChanged(newTrack) {
  return current_song.title !== newTrack.title || 
         current_song.artist !== newTrack.artist || 
         current_song.cover_art !== newTrack.cover_art;
}

function spotify_media(track) {
  if (!hasTrackChanged(track)) {
    return;
  }

  expand_pill();
  current_song = track;

  setTimeout(() => {
    if (track.player_name === 'Spotify') {
      contentContainer.innerHTML = `
        <dotlottie-wc
          src="https://lottie.host/daed205f-d595-472a-9ae2-3c3059d7f56a/tH6cS679WV.lottie"
          style="width: 50px;height: 50px"
          autoplay
          loop
        ></dotlottie-wc>

        <div class="right">
          <h3 class="name">${escapeHtml(track.title)}</h3>
          <img src="${track.cover_art}" alt="${escapeHtml(track.title)}-${escapeHtml(track.artist)}">
        </div>`;
    } else {
      contentContainer.innerHTML = `
        <dotlottie-wc
          src="https://lottie.host/daed205f-d595-472a-9ae2-3c3059d7f56a/tH6cS679WV.lottie"
          style="width: 50px;height: 50px"
          autoplay
          loop
        ></dotlottie-wc>

        <div class="right">
          <h3 class="name">${escapeHtml(track.title)}</h3>
          <div class="rightest-right">
            <span>${escapeHtml(track.artist)}</span>
          </div>
        </div>`;
    }

    const tl = gsap.timeline();
    tl.from(".content .right .name", {
      opacity: 0
    }, "together");
    tl.from(".content dotlottie-wc", {
      opacity: 0
    }, "together");
    tl.from(".content .right img, .content .right .rightest-right", {
      opacity: 0
    }, "together");
  }, 1400);
}

async function updateMediaStatus() {
  try {
    const track = await window.__TAURI__.core.invoke("spotify_now_playing");
    
    if (track && track.is_playing === true) {
      spotify_media(track);
    } else {
      if (current_song.title !== "") {
        collapse_pill();
      }
    }
  } catch (error) {
    console.error('Error fetching media:', error);
    if (current_song.title !== "") {
      collapse_pill();
    }
  }
}

startIdleClock();

setInterval(updateMediaStatus, 1000);