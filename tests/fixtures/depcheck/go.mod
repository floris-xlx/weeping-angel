module example.com/fixture

go 1.22

require (
        github.com/stretchr/testify v1.9.0
        example.com/internal/acme-go-sdk-xyz v1.2.3
)

replace example.com/internal/acme-go-sdk-xyz => ../acme-go-sdk-xyz
