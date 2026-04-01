import * as React from 'react'
import { Button, type ButtonProps } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useEmailConfigStore } from '@/stores/emailConfigStore'

interface EmailGuardButtonProps extends ButtonProps {
  requiresEmail?: boolean
}

const EmailGuardButton = React.forwardRef<HTMLButtonElement, EmailGuardButtonProps>(
  ({ requiresEmail = true, disabled, children, ...props }, ref) => {
    const emailEnabled = useEmailConfigStore((s) => s.emailEnabled)
    const emailDisabled = requiresEmail && !emailEnabled

    if (emailDisabled) {
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-block">
              <Button ref={ref} {...props} disabled style={{ pointerEvents: 'none' }}>
                {children}
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Email is not configured</TooltipContent>
        </Tooltip>
      )
    }

    return (
      <Button ref={ref} {...props} disabled={disabled}>
        {children}
      </Button>
    )
  }
)
EmailGuardButton.displayName = 'EmailGuardButton'

export { EmailGuardButton }
