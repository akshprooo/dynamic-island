const islandContainer = document.querySelector("#island");
const contentContainer = document.querySelector('.content');

// load
gsap.from(islandContainer, {
  opacity: 0,
});

function expand_pill(){
  gsap.to(islandContainer, {
    width:"200px",
    ease: 'elastic.out(.5, 0.3)',
    // ease:'back.out(1, 0.5)',
    duration:1.4
  })
}

function collapse_pill(){
  contentContainer.innerHTML = '';
  gsap.to(islandContainer, {
    width:'40px',
    ease: 'elastic.out(.1, 0.7)',
    duration:1.4
  })
}


let current_song = {title:"", artist:"", cover_art:""};
function spotify_media(track){
  if (current_song.title == track.title || current_song.artist == track.artist || current_song.cover_art == track.cover_art){
    
  }
  else{
    expand_pill();

    setTimeout(()=>{
      contentContainer.innerHTML = `<dotlottie-wc
                src="https://lottie.host/daed205f-d595-472a-9ae2-3c3059d7f56a/tH6cS679WV.lottie"
                style="width: 50px;height: 50px"
                autoplay
                loop
              ></dotlottie-wc>

              <div class="right">
                <h3 class="name">${track.title}</h3>
                <img src="${track.cover_art}" alt="${track.title}-${track.artist}">
              </div>`;
      const tl = gsap.timeline();  
      tl.from(".content .right .name", {
        opacity:0
      }, "together");
      tl.from(".content dotlottie-wc", {
        opacity:0
      }, "together")
      tl.from(".content .right img", {
        opacity:0
      }, "together")
    }, 1400)
    current_song = track;
  }
}


// spotify_media("Ishqa Ve", "Zeesha Ali", "https://images.lyricstelling.com/2025/08/ishqa-ve-zeeshan-ali-500x500.webp")
setInterval(async () => {
  const track = await window.__TAURI__.core.invoke("spotify_now_playing");
  if (track.is_playing==true){
    spotify_media(track);
  }else{
    collapse_pill();
  }
}, 1000);
