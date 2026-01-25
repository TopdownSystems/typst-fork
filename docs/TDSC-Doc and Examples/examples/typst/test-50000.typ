// Performance test: 50,000 words with masthead
// Tests that spilled paragraphs are correctly re-laid out at full width
// on pages without mastheads.
#set page(margin: 1in)
#set par(justify: true)

#masthead(right, 100pt)[Sidebar]

#lorem(50000)
