const axios = require("axios");
const dedent = require("dedent");
const getEmoji = require("get-random-emoji");
const scriptjs = require("scriptjs");
const wikidata = require("wikidata-sdk");

let rows = {};

const randomChoice = list => list[Math.floor(Math.random() * list.length)]

const randomProperty = f => f(randomChoice(rows))

const randomTriple = f => {
  randomProperty(([propertyID, propertyName]) => {
    // id properties aren't very fun; we will (try to) forbid them
    for (let word of ["code", "id", "identifier", "slug"])
      if (propertyName.toLowerCase().includes(word)) {
        randomTriple(f)
        return
      }

    const query = `
      SELECT ?aLabel ?bLabel
      WHERE {
        ?a wdt:${propertyID} ?b.
        SERVICE wikibase:label { bd:serviceParam wikibase:language "en". }
      }
      LIMIT 100`

    axios.get(wikidata.sparqlQuery(query)).then((res) => {
      const choices = res.data.results.bindings

      if (choices.length > 0) {
        const choice = randomChoice(choices)
        const [a, b] = [choice.aLabel.value, choice.bLabel.value];

        // unnamed objects aren't very fun either
        // (such objects have labels like Q123456789)
        if (!isNaN(parseInt(a.slice(1)))) {
          randomTriple(f)
          return
        }

        if (b.startsWith("http://") || b.startsWith("https://"))
          b = "<a href='" + b + "'>[link]</a>"

        f(a, propertyName, b)
      }
      else randomTriple(f)
    })
  })
}

function setFunFact(a, p, b) {
  let pFirstWord = p.split(" ")[0];
  if (pFirstWord.endsWith("ed") || p.endsWith(" of") || p.endsWith(" to")) {
    var prefix = "is ";
  } else if (pFirstWord.endsWith("s")) {
    var prefix = "";
  } else {
    var prefix = "has ";
  }

  document.getElementById(
    "fun-fact"
  ).innerHTML = `${a} <b>${prefix}${p}</b> ${b}`;
}

function reloadFunFact() {
  document.getElementById("fun-fact").innerHTML = "<i>loading…</i>";
  randomTriple(setFunFact);
}

function coinflip() {
  document.getElementById("coinflip").innerHTML = randomChoice(
    [
      "heads!",
      "tails!",
      "it lands on its side",
      "you lose the coin",
      "you place the coin in a vending machine, scoring yourself a coke",
      "the outcome you were hoping for",
      "the outcome you were dreading",
      "the coin never lands, instead bouncing with increasing intensity",
      "the coin never lands, instead hovering ominously",
      "the coin falls from the empire state building and pierces a woman's head, injuring her fatally",
      "the coin is covered in grime, obscuring which side is which",
      "the coin lands on a landmine, thereby curing your indecision",
      "the coin flips you off",
      "quicksand is not an ideal coin-flipping surface",
      "a seagull snatches the coin in mid-air",
      "a celeste snatches the coin in mid-air",
      "natural 1...",
      "natural 20!",
      "yahtzee!",
      "[REDACTED]",
      "<img src='https://izbicki.me/img/uploads/2011/11/coins-all.jpg' />",
    ]
  );
}

document.getElementById("celeste").innerHTML += " " + getEmoji()

document.getElementById("coelostat").innerHTML = randomChoice([
  "coelostat",
  "coelostat llc",
  "coelostat institute",
  "CIS",
  "infohazards",
  "infohazards.org",
  "pricing",
  "services"
]);

scriptjs(
  "https://cdn.jsdelivr.net/npm/jaaulde-cookies/lib/jaaulde-cookies.min.js",
  () => {
    var visits = parseInt(cookies.get("visits"));
    if (isNaN(visits)) visits = 1;
    else visits++;
    cookies.set("visits", visits);

    if (visits < 1) {
      var visitMessage = `you have apparently visited this site ${visits} times.`;
    } else if (visits === 1) {
      var visitMessage = "you have visited this site 1 time. welcome!";
    } else if (visits < 5) {
      var visitMessage = `you have visited this site ${visits} times. that is a normal amount.`;
    } else if (visits < 25) {
      var visitMessage = `you have visited this site ${visits} times. are you procrastinating?`;
    } else {
      var visitMessage = `you have visited this site ${visits} times. this is getting creepy!`;
    }

    document.getElementById("subheader").innerHTML = randomChoice([
      visitMessage,
      visitMessage,
      visitMessage,
      "you have lost the game!",
      "you have lost the game!",
      "you are now in control of your blinking!",
      "you are now in control of your breathing!",
      "you may now attend to that itch you've been neglecting!",
    ]);
  }
);

var allLinksVisible = false;
function toggleLinks() {
  allLinksVisible = !allLinksVisible;
  document.getElementById("more-links").style.display = allLinksVisible ? "block" : "none";
  document.getElementById("show-more-links").innerHTML = allLinksVisible ? "hide graveyard…" : "show graveyard…";
  document.getElementById("show-more-links-plus-minus").innerHTML = allLinksVisible ? "-" : "+";
}
document.getElementById("show-more-links").onclick = toggleLinks;

// function randomizeCSS() {
//   document.getElementById("is-this-website-ugly").innerHTML = "(randomizing…)";
// 
//   var randomComment = randomChoice(
//     ["/* hmm, maybe this will work? */\n",
//       "/* colors!!! */\n",
//       "/* now it's time to get serious */\n",
//       "/* now it's time to get serious */\n",
//     ]
//   )
//   var randomElement = randomChoice([
//     "* {",
//     "html {",
//     "body {",
//     "div {",
//     "h1 {",
//     "p {",
//     "a {",
//     "button {",
//   ])
//   var promptEnding = randomComment + randomElement;
//   var cssPrompt = dedent`
//   body {
//     font-family: "Libre Baskerville";
//     line-height: 1.6em;
// 
//     max-width: 60rem;
//     margin: 2em auto;
//     padding: 0 1em;
//   }
// 
//   @media screen and (max-width: 600px) {
//     body { font-size: 0.8em; }
//   }
// 
//   a { text-decoration: none; }
// 
//   /* usually sup affects line height */
//   sup {
//     vertical-align: top;
//     position: relative;
//     top: -0.5em;
//   }
// 
//   textarea {
//     font-family: "Source Code Pro";
//     padding: 0.5em;
//   }
// 
//   /* now let's make it ugly */
//   ` + promptEnding;
//   axios.post("/api/copilot", {
//     prompt: cssPrompt,
//     max_tokens: 200,
//     temperature: 1.7,
//   }).then((res) => {
//     document.getElementById("random-css").innerHTML = promptEnding + res.data.output;
//     document.getElementById("is-this-website-ugly").innerHTML = "is this website ugly?";
//   });
// }
// document.getElementById("randomize-css").onclick = randomizeCSS;

document.getElementById("flip-a-coin").onclick = coinflip

document.getElementById("dont-go-home").onclick = () => {
  document.getElementById("dont-go-home-text").innerHTML = "you're already here, idiot"
  document.getElementById("dont-go-home-text").style.opacity = 1.0
  setTimeout(
    () => {
      let opacity = 1.0
      let interval = setInterval(
        () => {
          opacity -= 0.01
          if (opacity < 0) {
            clearInterval(interval)
            opacity = 0
          }
          document.getElementById("dont-go-home-text").style.opacity = opacity
        },
        7,
      )
    },
    1250,
  )
}

document.getElementById("submit-feedback").onclick = () => {
  const feedback = document.getElementById("feedback-box").value
  axios.post("/api/feedback", { feedback })
  document.getElementById("feedback-box").value = ""
}

axios.get("https://quarry.wmcloud.org/run/45013/output/1/json").then((res) => {
  // wait till the data is loaded to enable the reload button and load the initial triple
  rows = res.data.rows;

  randomTriple(setFunFact);
  document.getElementById("reload-fun-fact").onclick = reloadFunFact;
});
