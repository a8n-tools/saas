export function TermsOfServicePage() {
  return (
    <div className="container max-w-4xl py-12">
      <h1 className="text-4xl font-bold mb-8">Terms of Service</h1>
      <p className="text-muted-foreground mb-8">Last updated: January 2025</p>

      <div className="prose prose-gray dark:prose-invert max-w-none space-y-8">
        <section>
          <h2 className="text-2xl font-semibold mb-4">1. Acceptance of Terms</h2>
          <p className="text-muted-foreground">
            By accessing or using a8n.tools ("the Service"), you agree to be bound by these Terms of Service.
            If you do not agree to these terms, please do not use the Service.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">2. Description of Service</h2>
          <p className="text-muted-foreground">
            a8n.tools provides a membership-based platform offering access to developer productivity tools,
            including but not limited to RUS (URL Shortener) and Rusty Links (Bookmark Management).
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">3. Membership and Payment</h2>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li>The Service is offered at a flat rate of $3/month for all applications.</li>
            <li>Early adopters who join at this rate will have their price locked for life.</li>
            <li>Payments are processed through Stripe. By joining, you also agree to Stripe's terms of service.</li>
            <li>Memberships renew automatically each month unless cancelled.</li>
            <li>You may cancel your membership at any time through your account settings.</li>
            <li>Upon cancellation, you retain access until the end of your current billing period.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">4. Grace Period</h2>
          <p className="text-muted-foreground">
            If a payment fails, you will be granted a 30-day grace period during which you retain full access
            to the Service. If payment is not resolved within this period, your membership will be cancelled
            and access will be revoked.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">5. User Accounts</h2>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li>You must provide accurate and complete information when creating an account.</li>
            <li>You are responsible for maintaining the security of your account credentials.</li>
            <li>You must notify us immediately of any unauthorized use of your account.</li>
            <li>One membership per individual. Account sharing is not permitted.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">6. Acceptable Use</h2>
          <p className="text-muted-foreground mb-4">You agree not to:</p>
          <ul className="list-disc pl-6 space-y-2 text-muted-foreground">
            <li>Use the Service for any illegal or unauthorized purpose.</li>
            <li>Attempt to gain unauthorized access to any part of the Service.</li>
            <li>Interfere with or disrupt the Service or servers connected to it.</li>
            <li>Use the Service to transmit malware, spam, or other harmful content.</li>
            <li>Violate any applicable laws or regulations.</li>
            <li>Abuse the Service resources (excessive API calls, storage abuse, etc.).</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">7. Intellectual Property</h2>
          <p className="text-muted-foreground">
            The Service and its original content, features, and functionality are owned by a8n.tools and are
            protected by international copyright, trademark, and other intellectual property laws. Our applications
            are built on open-source software, and we respect and comply with all applicable open-source licenses.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">8. Data and Privacy</h2>
          <p className="text-muted-foreground">
            Your use of the Service is also governed by our Privacy Policy. By using the Service, you consent
            to the collection and use of information as described in our Privacy Policy.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">9. Limitation of Liability</h2>
          <p className="text-muted-foreground">
            a8n.tools shall not be liable for any indirect, incidental, special, consequential, or punitive
            damages, including loss of profits, data, use, or other intangible losses, resulting from your
            access to or use of the Service. Our total liability shall not exceed the amount you paid for the
            Service in the twelve months prior to the claim.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">10. Disclaimer of Warranties</h2>
          <p className="text-muted-foreground">
            The Service is provided "as is" and "as available" without warranties of any kind, either express
            or implied. We do not warrant that the Service will be uninterrupted, error-free, or completely secure.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">11. Termination</h2>
          <p className="text-muted-foreground">
            We reserve the right to suspend or terminate your access to the Service at any time, with or without
            cause, with or without notice. Upon termination, your right to use the Service will immediately cease.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">12. Changes to Terms</h2>
          <p className="text-muted-foreground">
            We reserve the right to modify these Terms at any time. We will notify users of any material changes
            via email or through the Service. Your continued use of the Service after such modifications constitutes
            acceptance of the updated Terms.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold mb-4">13. Contact</h2>
          <p className="text-muted-foreground">
            For any questions about these Terms of Service, please contact us at{' '}
            <a href="mailto:support@a8n.tools" className="text-primary hover:underline">
              support@a8n.tools
            </a>
            .
          </p>
        </section>
      </div>
    </div>
  )
}
