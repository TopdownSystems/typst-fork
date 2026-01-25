// Performance test: 50,000 words with masthead
// Note: Due to known limitation with paragraph spill (see README.md section 4),
// paragraphs that start on the masthead page and span to subsequent pages
// will retain their narrow width. Using smaller paragraphs avoids this.
#set page(margin: 1in)
#set par(justify: true)

#masthead(right, 100pt)[Sidebar]

// Break into multiple paragraphs to avoid spill issue
#for i in range(100) {
  lorem(500)
  parbreak()
}
