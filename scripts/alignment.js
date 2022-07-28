rw = new WordPOS({ dictPath: "https://cdn.jsdelivr.net/npm/wordpos-web@1.0.2/dict" })

const generateAlignmentChart = () => {
  for (let prefix of ["col", "row"])
    for (let i = 1; i < 4; i++)
      rw.randAdjective(
        { count: 1 },
        (words, _) => document.getElementById(prefix + i).textContent = words[0].replace(/_/g, " "),
      )
}

document.getElementById("generate-alignment").onclick = generateAlignmentChart
