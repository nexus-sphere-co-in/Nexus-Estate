import { Box } from '@chakra-ui/layout';
import Template from '../src/template/template';
import { Wallet } from '../src/sections/wallet';

const WalletPage = () => {
    return (
        <Template
            section="wallet"
            element={[
                <Box h="full" key="walletbox">
                    <Wallet key="walletcomponent" />
                </Box>,
            ]}
        ></Template>
    );
};

export default WalletPage;
