var axios = require("axios");

function getCopilot() {
  document.getElementById("output-body").innerHTML = "[loading...]";

  var promptText = document.getElementById("prompt").value
  var maxTokens = parseInt(document.getElementById("max-tokens").value)
  var temperature = document.getElementById("temperature").value
  var topP = document.getElementById("top-p").value

  if (isNaN(maxTokens) || maxTokens < 1) {
    document.getElementById("output-body").innerHTML = "[invalid max tokens]"
    return
  }
  if (temperature === "") {
    temperature = 0.5
  } else {
    temperature = parseFloat(temperature)
    if (isNaN(temperature) || temperature < 0) {
      document.getElementById("output-body").innerHTML = "[invalid temperature]"
      return
    }
  }
  if (topP === "") {
    topP = 0.9
  } else {
    topP = parseFloat(topP)
    if (isNaN(topP) || topP < 0 || topP > 1) {
      document.getElementById("output-body").innerHTML = "[invalid top p]"
      return
    }
  }

  axios.post("/api/copilot", {
    prompt: promptText,
    max_tokens: maxTokens,
    temperature: temperature,
    top_p: topP
  }).then(function(response) {
    document.getElementById("output-body").innerHTML =
      `<b>${new Option(promptText).innerHTML}</b>${new Option(response.data.output).innerHTML}`
  }).catch(function(error) {
    document.getElementById("output-body").innerHTML = "[error]"
  });
}

document.getElementById("submit").onclick = getCopilot;
