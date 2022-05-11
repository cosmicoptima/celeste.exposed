# create a poll

create a twitter poll with more than four options

<p id="poll-error" style="color: #c00;"></p>

<div id="poll-options">
  <div class="poll-row"><input id="option-1" placeholder="option 1"></input></div>
  <div class="poll-row"><input id="option-2" placeholder="option 2"></input></div>
</div>
<div class="poll-row"><button id="add-option">+</button>&nbsp;<button id="create-poll">submit</button></div>

<p id="poll-success"></p>

<script>
var numOptions = 2

function addOption() {
  numOptions++
  let element = document.createElement("div")
  element.classList.add("poll-row")
  element.innerHTML = `<input id="option-${numOptions}" placeholder="option ${numOptions}"/>`
  document.getElementById("poll-options").appendChild(element)
}

function createPoll() {
  var options = []
  for (let i = 1; i <= numOptions; i++) {
    options.push(document.getElementById(`option-${i}`).value)
  }
  for (option in options) {
    if (options[option] === "") {
       document.getElementById("poll-error").innerHTML = "option can't be empty"
       return
     }
  }
  
  var xhr = new XMLHttpRequest()
  xhr.open("POST", "/api/poll/create", true)
  xhr.setRequestHeader("Content-Type", "application/json")
  xhr.onreadystatechange = function () {
    if (xhr.readyState === 4) {
      var url = JSON.parse(xhr.response).url
      document.getElementById("poll-success").innerHTML = `paste the following link into your tweet: <a href="${url}">${url}</a>`
    }
  }
  xhr.send(JSON.stringify(options))
}

document.getElementById("add-option").onclick = addOption
document.getElementById("create-poll").onclick = createPoll
</script>
