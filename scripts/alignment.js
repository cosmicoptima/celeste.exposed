rw = new WordPOS({
  dictPath: 'https://cdn.jsdelivr.net/npm/wordpos-web@1.0.2/dict'
})

function generateAlignmentChart() {
  for (let i = 1; i < 4; i++) {
    rw.randAdjective({ count: 1 }, (words, _) => {
      document.getElementById("row" + i).textContent = words[0].replace(/_/g, " ")
    })
  }
  for (let i = 1; i < 4; i++) {
    rw.randAdjective({ count: 1 }, (words, _) => {
      document.getElementById("col" + i).textContent = words[0].replace(/_/g, " ")
    })
  }
}

document.getElementById("generate-alignment").onclick = generateAlignmentChart
