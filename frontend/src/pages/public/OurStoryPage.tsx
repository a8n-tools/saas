export function OurStoryPage() {
  return (
    <div className="container max-w-4xl py-12">
      <h1 className="text-4xl font-bold mb-8">Our Story</h1>

      <div className="prose prose-gray dark:prose-invert max-w-none space-y-8">
        <section>
          <h2 className="text-2xl font-semibold mb-4">Why now?</h2>
          <div className="space-y-4 text-muted-foreground">
            <p>
              The constant churn of companies changing their license from open source to
              open core/enterprise edition is too much. Every year, another project that
              developers relied on pulls the rug, re-licensing under restrictive terms and
              leaving the community scrambling for alternatives.
            </p>
            <p>
              AI and search engines can consume the documentation without giving back to
              the developers. The people who write the code, maintain the projects, and
              answer the questions see none of the value extracted from their work.
            </p>
            <p>
              People expect "FOSS" to provide everything, when it doesn't need to provide
              everything for "free". There's a middle ground between fully open and fully
              closed &mdash; we need to make that clearer.
            </p>
          </div>
        </section>

        <section>
          <p className="text-muted-foreground italic">
            More to come...
          </p>
        </section>
      </div>
    </div>
  )
}
