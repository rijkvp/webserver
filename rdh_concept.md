# RHC

@rijkvp's simple Html Concatenator

## Named dynamic
`{#my_key}`

## Files
`{@footer.rhc}`
`{@header.rhc}`

## Lists
`
<h2>Blog Posts</h2>
{^blog_posts
<article>
<h3>~post_name~</h3>
<a href="~post_link~">Read post</a>
</article>
}
`

