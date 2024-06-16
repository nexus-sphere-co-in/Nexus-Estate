import { Heading, Text, SimpleGrid, Box } from '@chakra-ui/layout';
import { useEffect, useState } from 'react';
import { FaEthereum } from 'react-icons/fa';
import { getAllERC20Balances } from '../utils/backend';
import { getWalletEth } from '../utils/db';
import Token from './token';

interface ERC20AssetsResponse {
    name: string;
    balance: string;
    address: string;
    symbol: string;
}
const ERC20Assets = () => {
    const [coins, setCoins] = useState([]);
    useEffect(() => {
        async function readBalances() {
            const wallet = getWalletEth();
            if (wallet == null) {
                return;
            }
            let temp = await getAllERC20Balances(wallet);
            setCoins(temp.balances);
        }
        readBalances();
    }, []);
    return (
        <Box h="auto">
            <Box w="full" p={5}>
                <Heading size="lg" color="white" textAlign="center">
                    ERC20 tokens
                </Heading>
                <Text py={2} textAlign="center">
                    All the registered ERC20 tokens on Evmos.me
                </Text>
            </Box>
            <SimpleGrid
                mt={2}
                columns={
                    coins.length == 1
                        ? [1]
                        : coins.length == 2
                        ? [1, 1, 2, 2, 2]
                        : coins.length == 3
                        ? [1, 1, 2, 3, 3]
                        : [1, 1, 2, 3, 4]
                }
                columnGap={[0, 0, 3, 3, 3]}
                rowGap={6}
                h="full"
                justifyItems="flex-start"
            >
                {coins.map((item: ERC20AssetsResponse) => (
                    <Token
                        Icon={FaEthereum}
                        name={item.name}
                        balance={item.balance}
                        key={item.address}
                        address={item.address}
                        symbol={item.symbol}
                        transfer={'cosmos'}
                    />
                ))}
            </SimpleGrid>
        </Box>
    );
    //     <Box h="auto">
    //         <Box w="full" p={5}>
    //             <Heading size="lg" color="white" textAlign="center">
    //                 ERC20 tokens
    //             </Heading>
    //             <Text py={2} textAlign="center">
    //                 All the registered ERC20 tokens on Evmos.me
    //             </Text>
    //         </Box>
    //         <SimpleGrid
    //             mt={2}
    //             columns={
    //                 coins.length == 1
    //                     ? [1]
    //                     : coins.length == 2
    //                     ? [1, 1, 2, 2, 2]
    //                     : coins.length == 3
    //                     ? [1, 1, 2, 3, 3]
    //                     : [1, 1, 2, 3, 4]
    //             }
    //             columnGap={[0, 0, 3, 3, 3]}
    //             rowGap={6}
    //             h="full"
    //             justifyItems="center"
    //         >
    //             {coins.map((item) => (
    //                 <Token
    //                     Icon={FaEthereum}
    //                     name={item.name}
    //                     balance={item.balance}
    //                     address={item.address}
    //                     symbol={item.symbol}
    //                     key={item.name}
    //                     transfer={'erc20'}
    //                 />
    //             ))}
    //         </SimpleGrid>
    //     </Box>
    // );
};

export default ERC20Assets;
