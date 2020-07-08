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
