function addCss(fileName) {
  var head = document.head;
  var link = document.createElement("link");

  link.type = "text/css";
  link.rel = "stylesheet";
  link.href = fileName;

  head.appendChild(link);
}

addCss(
  "https://fonts.googleapis.com/css2?family=Rubik:wght@400;700&display=swap"
);

// Force insert buttons to Amethyst API and Website

var btnFelid = document.getElementsByClassName("right-buttons")[0];

var btn = document.createElement("A");
btn.href = "https://docs.amethyst.rs/master/amethyst/";
btn.className = "am-btns";
btn.innerHTML = "API Reference"; // Insert text
btnFelid.appendChild(btn);

var btn2 = document.createElement("A");
btn2.href = "https://amethyst.rs/";
btn2.className = "am-btns";
btn2.innerHTML = "Amethyst Website"; // Insert text
btnFelid.appendChild(btn2);
