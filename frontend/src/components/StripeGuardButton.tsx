import * as React from 'react'
import { Button, type ButtonProps } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useStripeConfigStore } from '@/stores/stripeConfigStore'

interface StripeGuardButtonProps extends ButtonProps {
  requiresStripe?: boolean
}

const StripeGuardButton = React.forwardRef<HTMLButtonElement, StripeGuardButtonProps>(
  ({ requiresStripe = true, disabled, children, ...props }, ref) => {
    const stripeEnabled = useStripeConfigStore((s) => s.stripeEnabled)
    const stripeDisabled = requiresStripe && !stripeEnabled

    if (stripeDisabled) {
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-block">
              <Button ref={ref} {...props} disabled style={{ pointerEvents: 'none' }}>
                {children}
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Payment is not configured</TooltipContent>
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
StripeGuardButton.displayName = 'StripeGuardButton'

export { StripeGuardButton }
