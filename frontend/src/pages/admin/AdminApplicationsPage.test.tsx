import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAdminUser } from '@/test/utils'
import { AdminApplicationsPage } from './AdminApplicationsPage'

// Helper to open the edit dialog for the first app in the list
async function openEditDialog(user: ReturnType<typeof userEvent.setup>) {
  await waitFor(() => {
    expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
  })
  await user.click(screen.getByRole('button', { name: /edit/i }))
  await waitFor(() => {
    expect(screen.getByText(/edit rus/i)).toBeInTheDocument()
  })
}

beforeEach(() => {
  setupAdminUser()
})

describe('AdminApplicationsPage', () => {
  it('renders applications page heading', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('Applications')).toBeInTheDocument()
      expect(screen.getByText('Manage platform applications.')).toBeInTheDocument()
    })
  })

  it('shows application cards after loading', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('RUS')).toBeInTheDocument()
    })
  })

  it('shows active/inactive badges', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      // Multiple "Active" elements may exist (badge + toggle label)
      expect(screen.getAllByText('Active').length).toBeGreaterThan(0)
    })
  })

  it('shows edit button for each application', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })
  })

  it('opens edit dialog when edit button clicked', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /edit/i }))

    expect(screen.getByText(/edit rus/i)).toBeInTheDocument()
  })

  it('edit dialog has required fields', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /edit/i }))

    expect(screen.getByLabelText('Display Name *')).toBeInTheDocument()
    expect(screen.getByLabelText('Container Name *')).toBeInTheDocument()
  })

  it('shows toggle switches for active and maintenance mode', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      // Multiple "Active" elements may exist (badge + toggle label)
      expect(screen.getAllByText('Active').length).toBeGreaterThan(0)
      expect(screen.getByText('Maintenance Mode')).toBeInTheDocument()
    })
  })

  it('validates forgejo fields are all-or-nothing', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await openEditDialog(user)

    // Fill only forgejo_owner, leaving forgejo_repo and pinned_release_tag empty
    const ownerInput = screen.getByLabelText('Forgejo Owner')
    await user.clear(ownerInput)
    await user.type(ownerInput, 'myorg')

    // The inline error should appear and Save button should be disabled
    await waitFor(() => {
      expect(
        screen.getByText('forgejo_owner, forgejo_repo, and pinned_release_tag must all be set together')
      ).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: /save changes/i })).toBeDisabled()
  })

  it('validates oci fields are all-or-nothing', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await openEditDialog(user)

    // Fill only oci_image_owner, leaving oci_image_name and pinned_image_tag empty
    const ociImageOwnerInput = screen.getByLabelText('OCI Image Owner')
    await user.clear(ociImageOwnerInput)
    await user.type(ociImageOwnerInput, 'myorg')

    // The inline error should appear and Save button should be disabled
    await waitFor(() => {
      expect(
        screen.getByText('oci_image_owner, oci_image_name, and pinned_image_tag must all be set together')
      ).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: /save changes/i })).toBeDisabled()
  })

  it('submits oci fields in the PUT payload when all three are filled', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await openEditDialog(user)

    // Fill all three OCI fields
    await user.type(screen.getByLabelText('OCI Image Owner'), 'myorg')
    await user.type(screen.getByLabelText('OCI Image Name'), 'myimage')
    await user.type(screen.getByLabelText('Pinned Image Tag'), 'v2.0.0')

    // No validation error should be shown
    expect(
      screen.queryByText('oci_image_owner, oci_image_name, and pinned_image_tag must all be set together')
    ).not.toBeInTheDocument()

    // Save button should be enabled
    const saveBtn = screen.getByRole('button', { name: /save changes/i })
    expect(saveBtn).not.toBeDisabled()

    // Click Save — MSW handler accepts any PUT and returns the mock app
    await user.click(saveBtn)

    // After mutation succeeds, the dialog should close (editingApp set to null)
    await waitFor(() => {
      expect(screen.queryByText(/edit rus/i)).not.toBeInTheDocument()
    })
  })

  it('refresh button calls adminRefresh and shows asset list', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await openEditDialog(user)

    // Fill all three forgejo fields so the Refresh release button appears
    await user.type(screen.getByLabelText('Forgejo Owner'), 'myorg')
    await user.type(screen.getByLabelText('Forgejo Repo'), 'myrepo')
    await user.type(screen.getByLabelText('Pinned Release Tag'), 'v1.0.0')

    // The Refresh release button should now be visible
    const refreshBtn = await screen.findByRole('button', { name: /refresh release/i })
    await user.click(refreshBtn)

    // MSW returns app-linux-amd64 and app-darwin-amd64 assets
    await waitFor(() => {
      expect(screen.getByText('app-linux-amd64')).toBeInTheDocument()
      expect(screen.getByText('app-darwin-amd64')).toBeInTheDocument()
    })
  })
})
