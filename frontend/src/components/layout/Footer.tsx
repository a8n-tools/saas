import { Link } from 'react-router-dom'

export function Footer() {
  return (
    <footer className="border-t bg-background">
      <div className="container py-8 md:py-12">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          <div className="col-span-2 md:col-span-1">
            <Link to="/" className="flex items-center space-x-2">
              <span className="text-xl font-bold text-primary">a8n</span>
              <span className="text-xl font-light">.tools</span>
            </Link>
            <p className="mt-4 text-sm text-muted-foreground">
              Developer tools, automated.
            </p>
          </div>
          <div>
            <h3 className="text-sm font-semibold">Product</h3>
            <ul className="mt-4 space-y-3 text-sm">
              <li>
                <Link
                  to="/pricing"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Pricing
                </Link>
              </li>
              <li>
                <a
                  href="https://rus.a8n.tools"
                  className="text-muted-foreground hover:text-foreground"
                >
                  RUS
                </a>
              </li>
              <li>
                <a
                  href="https://rustylinks.a8n.tools"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Rusty Links
                </a>
              </li>
            </ul>
          </div>
          <div>
            <h3 className="text-sm font-semibold">Account</h3>
            <ul className="mt-4 space-y-3 text-sm">
              <li>
                <Link
                  to="/login"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Login
                </Link>
              </li>
              <li>
                <Link
                  to="/register"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Register
                </Link>
              </li>
            </ul>
          </div>
          <div>
            <h3 className="text-sm font-semibold">Legal</h3>
            <ul className="mt-4 space-y-3 text-sm">
              <li>
                <Link
                  to="/terms"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Terms of Service
                </Link>
              </li>
              <li>
                <Link
                  to="/privacy"
                  className="text-muted-foreground hover:text-foreground"
                >
                  Privacy Policy
                </Link>
              </li>
            </ul>
          </div>
        </div>
        <div className="mt-8 border-t pt-8 text-center text-sm text-muted-foreground">
          <p>&copy; {new Date().getFullYear()} a8n.tools. All rights reserved.</p>
        </div>
      </div>
    </footer>
  )
}
