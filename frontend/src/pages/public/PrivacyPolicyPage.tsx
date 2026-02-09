export function PrivacyPolicyPage() {
  return (
    <div className="container max-w-4xl py-12">
      <h1 className="text-4xl font-bold mb-8">Privacy Policy</h1>
      <p className="text-muted-foreground mb-8">Last updated: January 2025</p>

      <div className="prose prose-gray dark:prose-invert max-w-none space-y-8">
        <section>
          <h2 className="text-2xl font-semibold mb-4">1. Introduction</h2>
          <p className="text-muted-foreground">
            This Privacy Policy describes how a8n.tools ("we", "our", or "us") collects, uses, and protects your
            personal information when you use our services. We are committed to protecting your privacy and handling
            your data transparently.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">2. Information We Collect</h2>

          <h3 className="text-xl font-medium mt-6 mb-3">2.1 Information You Provide</h3>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li><strong>Account Information:</strong> Email address, password (hashed), and account preferences.</li>
            <li><strong>Payment Information:</strong> Processed securely through Stripe. We do not store full credit card numbers.</li>
            <li><strong>Application Data:</strong> Data you create within our applications (URLs, bookmarks, etc.).</li>
          </ul>

          <h3 className="text-xl font-medium mt-6 mb-3">2.2 Information Collected Automatically</h3>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li><strong>Usage Data:</strong> How you interact with our services, features used, and actions taken.</li>
            <li><strong>Device Information:</strong> Browser type, operating system, and device identifiers.</li>
            <li><strong>Log Data:</strong> IP addresses, access times, and pages visited.</li>
            <li><strong>Cookies:</strong> Session cookies for authentication and preferences.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">3. How We Use Your Information</h2>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li>To provide and maintain our services.</li>
            <li>To process payments and manage memberships.</li>
            <li>To communicate with you about your account and services.</li>
            <li>To send important updates and security notices.</li>
            <li>To improve our services and develop new features.</li>
            <li>To prevent fraud and ensure security.</li>
            <li>To comply with legal obligations.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">4. Data Sharing</h2>
          <p className="text-muted-foreground mb-4">
            We do not sell your personal information. We may share data with:
          </p>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li><strong>Stripe:</strong> For payment processing.</li>
            <li><strong>Service Providers:</strong> Who help us operate our services (hosting, email, monitoring).</li>
            <li><strong>Legal Requirements:</strong> When required by law or to protect our rights.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">5. Data Security</h2>
          <p className="text-muted-foreground">
            We implement industry-standard security measures to protect your data:
          </p>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground mt-4">
            <li>Passwords are hashed using Argon2id.</li>
            <li>All connections are encrypted with TLS/SSL.</li>
            <li>Authentication tokens are cryptographically signed.</li>
            <li>Regular security audits and monitoring.</li>
            <li>Access controls and audit logging.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">6. Data Retention</h2>
          <p className="text-muted-foreground">
            We retain your data for as long as your account is active or as needed to provide services. After account
            deletion, we may retain certain data for legal compliance, dispute resolution, or legitimate business
            purposes for a limited period.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">7. Your Rights</h2>
          <p className="text-muted-foreground mb-4">You have the right to:</p>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li><strong>Access:</strong> Request a copy of your personal data.</li>
            <li><strong>Correction:</strong> Request correction of inaccurate data.</li>
            <li><strong>Deletion:</strong> Request deletion of your account and data.</li>
            <li><strong>Portability:</strong> Request your data in a portable format.</li>
            <li><strong>Objection:</strong> Object to certain data processing.</li>
          </ul>
          <p className="text-muted-foreground mt-4">
            To exercise these rights, contact us at{' '}
            <a href="mailto:privacy@a8n.tools" className="text-primary hover:underline">
              privacy@a8n.tools
            </a>
            .
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">8. Cookies</h2>
          <p className="text-muted-foreground">
            We use essential cookies for authentication and session management. These cookies are necessary for the
            Service to function and cannot be disabled. We do not use third-party tracking cookies or advertising cookies.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">9. Third-Party Services</h2>
          <p className="text-muted-foreground">
            Our Service may contain links to third-party websites. We are not responsible for the privacy practices
            of these external sites. We encourage you to review their privacy policies.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">10. Children's Privacy</h2>
          <p className="text-muted-foreground">
            Our Service is not intended for users under 13 years of age. We do not knowingly collect personal
            information from children under 13. If we become aware of such collection, we will delete the data promptly.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">11. International Data Transfers</h2>
          <p className="text-muted-foreground">
            Your data may be processed in countries other than your own. We ensure appropriate safeguards are in
            place for any international transfers.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">12. Changes to This Policy</h2>
          <p className="text-muted-foreground">
            We may update this Privacy Policy from time to time. We will notify you of significant changes via email
            or through our Service. Your continued use of the Service after changes indicates acceptance of the updated policy.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">13. Contact Us</h2>
          <p className="text-muted-foreground">
            For questions about this Privacy Policy or our data practices, contact us at:
          </p>
          <ul className="list-none space-y-1 text-muted-foreground mt-4">
            <li>
              Email:{' '}
              <a href="mailto:privacy@a8n.tools" className="text-primary hover:underline">
                privacy@a8n.tools
              </a>
            </li>
            <li>
              General inquiries:{' '}
              <a href="mailto:support@a8n.tools" className="text-primary hover:underline">
                support@a8n.tools
              </a>
            </li>
          </ul>
        </section>
      </div>
    </div>
  )
}
