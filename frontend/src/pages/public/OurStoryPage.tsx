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
          <h2 className="text-2xl font-semibold mb-4">A different model</h2>
          <div className="space-y-4 text-muted-foreground">
            <p>
              Most open source businesses follow the same playbook: release
              a free "community edition" and gate the best features behind
              an enterprise paywall. The community gets free support through
              GitHub issues and forums while the business monetises the
              software itself.
            </p>
            <p>
              a8n Tools flips that model 90&deg;. Every feature ships free,
              to everyone, forever. What you pay for is the <em>community</em>
              &mdash; the maintenance, the support, the people behind the
              project.
            </p>
          </div>

          <div className="mt-6 overflow-x-auto">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border border-border p-3 bg-muted/50 text-left font-semibold text-foreground">
                    Typical open source
                  </th>
                  <th className="border border-primary/30 p-3 bg-primary/10 text-left font-semibold text-foreground">
                    a8n Tools
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td className="border border-border p-4 bg-muted text-foreground">
                    Open core / Open source
                  </td>
                  <td className="border border-primary/30 p-4 bg-primary/10 text-foreground">
                    Free community support (GitHub issues, forums)
                  </td>
                </tr>
                <tr>
                  <td className="border border-border p-4 bg-muted text-foreground">
                    Enterprise features / Closed source
                  </td>
                  <td className="border border-primary/30 p-4 bg-primary/10 text-foreground">
                    Paid support for customers
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div className="mt-6 space-y-4 text-muted-foreground">
            <p>
              This follows the{' '}
              <a
                href="https://opensourcemaintenancefee.org/maintainers/"
                target="_blank"
                rel="noopener noreferrer"
                className="underline hover:text-foreground"
              >
                Open Source Maintenance Fee
              </a>
              {' '}methodology, which draws a clear line between the <em>Software</em> and
              the <em>Project</em>. The source code is free &mdash; as in freedom &mdash; but
              the maintenance work is not. You collaborate on the Software and you get paid to
              maintain the Project.
            </p>
            <p>
              When a typical open source company needs more revenue, they move features from
              free to paid. When a8n Tools needs more revenue, we build more software and
              make the community more valuable. Our incentives point the same direction as
              yours.
            </p>
          </div>
        </section>
      </div>
    </div>
  )
}
