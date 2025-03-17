// Selecting elements
const heading = document.getElementById("main-heading");
const paragraphs = document.getElementsByTagName("p");
const buttons = document.querySelectorAll(".btn");

// Modifying elements
heading.textContent = "Welcome to My Website";
paragraphs[0].innerHTML = "This is the <strong>first</strong> paragraph.";
buttons.forEach(button => {
    button.style.backgroundColor = "blue";
    button.style.color = "white";
});

// Creating and appending elements
const newDiv = document.createElement("div");
newDiv.className = "info-box";
newDiv.textContent = "This is a new div element.";
document.body.appendChild(newDiv);
